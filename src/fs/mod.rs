//! File-system tools: read/write/edit files and directories with optional sandbox.
//!
//! Use [`FsContext`] to set the workspace root (canonical). Relative paths are always resolved
//! against that root. When [`FsContext::allow_outside_root`] is `false`, every resolved path must
//! stay under the root; when `true`, that check is omitted (absolute paths and `..` normalization
//! may therefore leave the workspace).

mod error;
mod ops;
mod tools;

use std::path::PathBuf;

pub use tools::{
    all_tools, CopyFileTool, CreateDirectoryTool, DeleteFileTool, EditFileTool, ListDirectoryTool,
    MoveFileTool, ReadFileTool, WriteFileTool,
};

/// Shared settings for fs tools: canonical workspace root and sandbox mode.
#[derive(Debug, Clone)]
pub struct FsContext {
    /// Canonical workspace root: relative paths join against this directory.
    pub root_canonical: PathBuf,
    /// When `false`, resolved paths must remain under [`Self::root_canonical`]. When `true`, that
    /// boundary check is skipped (absolute paths and normalized paths may lie outside the root).
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
        assert_eq!(err.code, "PATTERN_NOT_UNIQUE");

        let err = edit
            .execute(json!({
                "path": "x.txt",
                "old_text": "zzz",
                "new_text": "q"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "PATTERN_NOT_FOUND");

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
        assert_eq!(err.code, "INVALID_PATH");
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
        assert_eq!(err.code, "FILE_ALREADY_EXISTS");
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn read_file_rejects_non_integer_offset_or_limit() {
        let root = tmp_root();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let write = tools.iter().find(|t| t.name() == "write_file").unwrap();
        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();
        write
            .execute(json!({ "path": "t.txt", "content": "a\nb\n" }))
            .await
            .unwrap();

        for params in [
            json!({ "path": "t.txt", "offset": 1.0 }),
            json!({ "path": "t.txt", "limit": 1.5 }),
        ] {
            let err = read.execute(params).await.unwrap_err();
            assert_eq!(err.code, "INVALID_PATH");
        }

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn read_file_schema_uses_integer_contract() {
        let root = tmp_root();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();
        let schema = read.schema();
        assert_eq!(schema["properties"]["offset"]["type"], "integer");
        assert_eq!(schema["properties"]["offset"]["minimum"], 1);
        assert_eq!(schema["properties"]["limit"]["type"], "integer");
        assert_eq!(schema["properties"]["limit"]["minimum"], 0);
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn list_directory_sorted_entries() {
        let root = tmp_root();
        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("b.txt"), "x").unwrap();
        fs::write(root.join("a.txt"), "y").unwrap();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let list = tools.iter().find(|t| t.name() == "list_directory").unwrap();
        let out = list.execute(json!({ "path": "." })).await.unwrap();
        assert_eq!(out["success"], true);
        let entries = out["data"]["entries"].as_array().unwrap();
        let names: Vec<_> = entries
            .iter()
            .map(|e| e["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "sub"]);
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn move_file_to_nested_destination() {
        let root = tmp_root();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let write = tools.iter().find(|t| t.name() == "write_file").unwrap();
        let mv = tools.iter().find(|t| t.name() == "move_file").unwrap();
        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();
        write
            .execute(json!({ "path": "src.txt", "content": "moved" }))
            .await
            .unwrap();
        let out = mv
            .execute(json!({
                "source": "src.txt",
                "destination": "nested/dst.txt"
            }))
            .await
            .unwrap();
        assert_eq!(out["success"], true);
        assert!(!root.join("src.txt").exists());
        let r = read
            .execute(json!({ "path": "nested/dst.txt" }))
            .await
            .unwrap();
        assert_eq!(r["data"]["content"], "moved");
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn read_file_preserves_crlf_and_trailing_newline() {
        let root = tmp_root();
        // Write CRLF content at the byte level — bypassing write_file so the \r\n is verbatim.
        std::fs::write(root.join("crlf.txt"), b"line1\r\nline2\r\n").unwrap();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();

        // Full read: CRLF must survive and trailing newline must be present.
        let r = read
            .execute(serde_json::json!({ "path": "crlf.txt" }))
            .await
            .unwrap();
        assert_eq!(r["data"]["content"], "line1\r\nline2\r\n");
        assert_eq!(r["data"]["total_lines"], 2);

        // Partial read (offset=1, limit=1): CRLF on the selected line is preserved.
        let r2 = read
            .execute(serde_json::json!({ "path": "crlf.txt", "offset": 1, "limit": 1 }))
            .await
            .unwrap();
        assert_eq!(r2["data"]["content"], "line1\r");

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn read_file_preserves_trailing_newline_lf() {
        let root = tmp_root();
        std::fs::write(root.join("t.txt"), b"hello\nworld\n").unwrap();
        let ctx = Arc::new(FsContext::new(Some(root.clone()), false).expect("ctx"));
        let tools = all_tools(ctx);
        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();

        let r = read
            .execute(serde_json::json!({ "path": "t.txt" }))
            .await
            .unwrap();
        assert_eq!(r["data"]["content"], "hello\nworld\n");
        assert_eq!(r["data"]["total_lines"], 2);

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
        assert_eq!(err.code, "INVALID_PATH");
        let _ = fs::remove_file(&sibling);
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn relaxed_mode_allows_absolute_path_outside_workspace_root() {
        let base = tmp_root();
        let workspace = base.join("workspace");
        let outside = base.join("outside");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let secret = outside.join("secret.txt");
        fs::write(&secret, "outside-data").unwrap();
        let secret_abs = secret.canonicalize().unwrap();
        let path_arg = secret_abs.to_string_lossy().to_string();

        let ctx_sandbox = Arc::new(FsContext::new(Some(workspace.clone()), false).unwrap());
        let tools_sandbox = all_tools(ctx_sandbox);
        let read = tools_sandbox
            .iter()
            .find(|t| t.name() == "read_file")
            .unwrap();
        let err = read
            .execute(json!({ "path": path_arg.clone() }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let ctx_relaxed = Arc::new(FsContext::new(Some(workspace.clone()), true).unwrap());
        let tools_relaxed = all_tools(ctx_relaxed);
        let read_r = tools_relaxed
            .iter()
            .find(|t| t.name() == "read_file")
            .unwrap();
        let r = read_r.execute(json!({ "path": path_arg })).await.unwrap();
        assert_eq!(r["data"]["content"], "outside-data");

        let _ = fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn relaxed_mode_relative_path_joins_workspace_root() {
        let base = tmp_root();
        let workspace = base.join("ws");
        fs::create_dir_all(&workspace).unwrap();
        let ctx = Arc::new(FsContext::new(Some(workspace.clone()), true).unwrap());
        let tools = all_tools(ctx);
        let write = tools.iter().find(|t| t.name() == "write_file").unwrap();
        let read = tools.iter().find(|t| t.name() == "read_file").unwrap();
        write
            .execute(json!({
                "path": "nested/relaxed.txt",
                "content": "in-ws"
            }))
            .await
            .unwrap();
        let on_disk = workspace.join("nested").join("relaxed.txt");
        assert!(on_disk.is_file());
        let r = read
            .execute(json!({ "path": "nested/relaxed.txt" }))
            .await
            .unwrap();
        assert_eq!(r["data"]["content"], "in-ws");
        let _ = fs::remove_dir_all(&base);
    }
}
