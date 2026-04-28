//! Local workspace find tools: [`grep_search`](GrepSearchTool) and [`glob_search`](GlobSearchTool).
//!
//! This module is named `find` to distinguish **local lookup** from network search (the `web` feature).
//! Use [`FindContext`] for the default scan root (canonical). Omitted `path` in tool calls starts
//! from that root. Relative `path` joins against it; absolute paths are blocked unless
//! [`FindContext::allow_outside_root`] is enabled.

mod error;
mod ops;
mod path;
mod tools;

use std::path::PathBuf;

pub use tools::{all_tools, GlobSearchTool, GrepSearchTool};

/// Default workspace root for find tools (canonical).
#[derive(Debug, Clone)]
pub struct FindContext {
    pub root_canonical: PathBuf,
    /// When `false`, resolved paths must remain under [`Self::root_canonical`]. When `true`,
    /// absolute paths and normalized relative paths may point outside the workspace root.
    pub allow_outside_root: bool,
}

impl FindContext {
    /// Create context. `root: None` uses [`std::env::current_dir`] at construction time.
    pub fn new(root: Option<PathBuf>) -> std::io::Result<Self> {
        Self::with_allow_outside_root(root, false)
    }

    /// Same as [`Self::new`] but optionally allows scanning outside the workspace root.
    pub fn with_allow_outside_root(
        root: Option<PathBuf>,
        allow_outside_root: bool,
    ) -> std::io::Result<Self> {
        let r = match root {
            Some(p) => p,
            None => std::env::current_dir()?,
        };
        Ok(Self {
            root_canonical: r.canonicalize()?,
            allow_outside_root,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use serde_json::json;

    use super::*;

    fn tmp_root() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "agentool_find_test_{}_{}",
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
    async fn grep_finds_lines_and_respects_glob() {
        let root = tmp_root();
        fs::write(root.join("a.md"), "hello world\nskip me\n").unwrap();
        fs::write(root.join("b.txt"), "hello rust\n").unwrap();
        let ctx = Arc::new(FindContext::new(Some(root.clone())).unwrap());
        let tools = all_tools(ctx);
        let grep = tools.iter().find(|t| t.name() == "grep_search").unwrap();

        let out = grep
            .execute(json!({
                "pattern": "hello",
                "glob": "*.md"
            }))
            .await
            .unwrap();
        assert_eq!(out["success"], true);
        let m = out["data"]["matches"].as_array().unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0]["line"], 1);
        assert_eq!(m[0]["content"], "hello world");
        assert_eq!(m[0]["file"], "a.md");

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn grep_ignore_case() {
        let root = tmp_root();
        fs::write(root.join("x.txt"), "Foo\n").unwrap();
        let ctx = Arc::new(FindContext::new(Some(root.clone())).unwrap());
        let tools = all_tools(ctx);
        let grep = tools.iter().find(|t| t.name() == "grep_search").unwrap();
        let out = grep
            .execute(json!({
                "pattern": "foo",
                "ignore_case": true
            }))
            .await
            .unwrap();
        assert_eq!(out["data"]["matches"].as_array().unwrap().len(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn grep_invalid_regex() {
        let root = tmp_root();
        let ctx = Arc::new(FindContext::new(Some(root.clone())).unwrap());
        let tools = all_tools(ctx);
        let grep = tools.iter().find(|t| t.name() == "grep_search").unwrap();
        let err = grep.execute(json!({ "pattern": "(" })).await.unwrap_err();
        assert_eq!(err.code, "INVALID_PATTERN");
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn glob_lists_sorted() {
        let root = tmp_root();
        fs::write(root.join("z.md"), "").unwrap();
        fs::write(root.join("a.md"), "").unwrap();
        let ctx = Arc::new(FindContext::new(Some(root.clone())).unwrap());
        let tools = all_tools(ctx);
        let glob = tools.iter().find(|t| t.name() == "glob_search").unwrap();
        let out = glob.execute(json!({ "pattern": "*.md" })).await.unwrap();
        let files: Vec<_> = out["data"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(files, vec!["a.md", "z.md"]);
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn find_path_missing_returns_file_not_found() {
        let root = tmp_root();
        let ctx = Arc::new(FindContext::new(Some(root.clone())).unwrap());
        let tools = all_tools(ctx);
        let glob = tools.iter().find(|t| t.name() == "glob_search").unwrap();
        let err = glob
            .execute(json!({
                "pattern": "*.md",
                "path": "nope"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "FILE_NOT_FOUND");
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn sandbox_blocks_absolute_path_outside_workspace_root() {
        let base = tmp_root();
        let workspace = base.join("workspace");
        let outside = base.join("outside");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.txt"), "secret").unwrap();

        let ctx = Arc::new(FindContext::new(Some(workspace.clone())).unwrap());
        let tools = all_tools(ctx);
        let glob = tools.iter().find(|t| t.name() == "glob_search").unwrap();
        let err = glob
            .execute(json!({
                "pattern": "*.txt",
                "path": outside.to_string_lossy().to_string()
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let _ = fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn relaxed_mode_allows_absolute_path_outside_workspace_root() {
        let base = tmp_root();
        let workspace = base.join("workspace");
        let outside = base.join("outside");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.txt"), "secret").unwrap();

        let ctx =
            Arc::new(FindContext::with_allow_outside_root(Some(workspace.clone()), true).unwrap());
        let tools = all_tools(ctx);
        let glob = tools.iter().find(|t| t.name() == "glob_search").unwrap();
        let out = glob
            .execute(json!({
                "pattern": "*.txt",
                "path": outside.to_string_lossy().to_string()
            }))
            .await
            .unwrap();
        let files = out["data"]["files"].as_array().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "secret.txt");
        assert_eq!(out["data"]["truncated"], false);

        let _ = fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn grep_and_glob_limit_results_and_mark_truncated() {
        let root = tmp_root();
        fs::write(root.join("a.txt"), "hit one\n").unwrap();
        fs::write(root.join("b.txt"), "hit two\n").unwrap();
        fs::write(root.join("c.txt"), "hit three\n").unwrap();
        let ctx = Arc::new(FindContext::new(Some(root.clone())).unwrap());
        let tools = all_tools(ctx);

        let grep = tools.iter().find(|t| t.name() == "grep_search").unwrap();
        let grep_out = grep
            .execute(json!({
                "pattern": "hit",
                "limit": 2
            }))
            .await
            .unwrap();
        assert_eq!(grep_out["data"]["matches"].as_array().unwrap().len(), 2);
        assert_eq!(grep_out["data"]["truncated"], true);

        let glob = tools.iter().find(|t| t.name() == "glob_search").unwrap();
        let glob_out = glob
            .execute(json!({
                "pattern": "*.txt",
                "limit": 2
            }))
            .await
            .unwrap();
        assert_eq!(glob_out["data"]["files"].as_array().unwrap().len(), 2);
        assert_eq!(glob_out["data"]["truncated"], true);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn schema_declares_limit_as_integer_contract() {
        let root = tmp_root();
        let ctx = Arc::new(FindContext::new(Some(root.clone())).unwrap());
        let tools = all_tools(ctx);

        let grep = tools.iter().find(|t| t.name() == "grep_search").unwrap();
        let grep_schema = grep.schema();
        assert_eq!(grep_schema["properties"]["limit"]["type"], "integer");
        assert_eq!(grep_schema["properties"]["limit"]["minimum"], 1);

        let glob = tools.iter().find(|t| t.name() == "glob_search").unwrap();
        let glob_schema = glob.schema();
        assert_eq!(glob_schema["properties"]["limit"]["type"], "integer");
        assert_eq!(glob_schema["properties"]["limit"]["minimum"], 1);

        let _ = fs::remove_dir_all(&root);
    }
}
