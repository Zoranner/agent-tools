//! File-system tools: read/write/edit files and directories with optional sandbox.
//!
//! Use [`FsContext`] to set the workspace root (canonical). When [`FsContext::allow_outside_root`]
//! is `false`, all resolved paths must stay under that root.

mod error_map;
mod ops;
mod path_policy;
mod tools;

use std::path::PathBuf;

pub use tools::{
    all_tools, CopyFileTool, CreateDirectoryTool, DeleteFileTool, EditFileTool, ListDirectoryTool,
    MoveFileTool, ReadFileTool, WriteFileTool,
};

/// Shared settings for fs tools: canonical workspace root and sandbox mode.
#[derive(Debug, Clone)]
pub struct FsContext {
    /// Canonical workspace root enforced when [`Self::allow_outside_root`] is `false`.
    pub root_canonical: PathBuf,
    /// When `true`, paths resolve from the process current directory without sandbox checks.
    pub allow_outside_root: bool,
}

impl FsContext {
    /// Create context. `root: None` uses [`std::env::current_dir`].
    pub fn new(root: Option<PathBuf>, allow_outside_root: bool) -> std::io::Result<Self> {
        let r = match root {
            Some(p) => p,
            None => std::env::current_dir()?,
        };
        let root_canonical = r.canonicalize()?;
        Ok(Self {
            root_canonical,
            allow_outside_root,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use serde_json::json;
    use std::fs;

    fn tmp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "agentool_fs_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).expect("create tmp");
        dir
    }

    #[tokio::test]
    async fn sandbox_read_write_roundtrip() {
        let root = tmp_root();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx.clone());
        let write = tools
            .iter()
            .find(|t| t.name() == "write_file")
            .expect("write_file");
        let read = tools
            .iter()
            .find(|t| t.name() == "read_file")
            .expect("read_file");

        let rel = "sub/a.txt";
        let w = write
            .execute(json!({
                "path": rel,
                "content": "line1\nline2\n"
            }))
            .await
            .expect("write");
        assert_eq!(w["success"], true);

        let r = read
            .execute(json!({ "path": rel, "offset": 2, "limit": 1 }))
            .await
            .expect("read");
        assert_eq!(r["success"], true);
        assert_eq!(r["data"]["content"], "line2");
        assert_eq!(r["data"]["total_lines"], 2);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn edit_file_unique_and_errors() {
        let root = tmp_root();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let write = tools.iter().find(|t| t.name() == "write_file").unwrap();
        let edit = tools.iter().find(|t| t.name() == "edit_file").unwrap();

        write
            .execute(json!({ "path": "x.txt", "content": "ababa" }))
            .await
            .unwrap();

        let err = edit
            .execute(json!({
                "path": "x.txt",
                "old_text": "a",
                "new_text": "z"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code.to_string(), "PATTERN_NOT_UNIQUE");

        let err = edit
            .execute(json!({
                "path": "x.txt",
                "old_text": "zzz",
                "new_text": "q"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code.to_string(), "PATTERN_NOT_FOUND");

        edit.execute(json!({
            "path": "x.txt",
            "old_text": "bab",
            "new_text": "BAB"
        }))
        .await
        .unwrap();

        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();
        let r = read.execute(json!({ "path": "x.txt" })).await.unwrap();
        assert_eq!(r["data"]["content"], "aBABa");

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn delete_file_rejects_directory() {
        let root = tmp_root();
        fs::create_dir(root.join("d")).unwrap();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let del = tools.iter().find(|t| t.name() == "delete_file").unwrap();
        let err = del.execute(json!({ "path": "d" })).await.unwrap_err();
        assert_eq!(err.code.to_string(), "INVALID_PATH");
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn copy_file_destination_exists() {
        let root = tmp_root();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let write = tools.iter().find(|t| t.name() == "write_file").unwrap();
        let copy = tools.iter().find(|t| t.name() == "copy_file").unwrap();
        write
            .execute(json!({ "path": "a.txt", "content": "x" }))
            .await
            .unwrap();
        write
            .execute(json!({ "path": "b.txt", "content": "y" }))
            .await
            .unwrap();
        let err = copy
            .execute(json!({ "source": "a.txt", "destination": "b.txt" }))
            .await
            .unwrap_err();
        assert_eq!(err.code.to_string(), "FILE_ALREADY_EXISTS");
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn sandbox_blocks_escape() {
        let root = tmp_root();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();
        let outside = root.parent().expect("parent");
        // Path clearly outside sandbox root (sibling directory in temp)
        let sibling = outside.join(format!("agentool_escape_probe_{}", std::process::id()));
        let _ = fs::write(&sibling, "secret");
        let err = read
            .execute(json!({ "path": sibling.to_string_lossy() }))
            .await
            .unwrap_err();
        assert_eq!(err.code.to_string(), "INVALID_PATH");
        let _ = fs::remove_file(&sibling);
        let _ = fs::remove_dir_all(&root);
    }
}
