//! Markdown analysis tools: [`extract_toc`](ExtractTocTool) and [`markdown_stats`](MarkdownStatsTool).
//!
//! Paths resolve against [`MdContext::root_canonical`] like `fs` tools; [`MdContext::allow_outside_root`]
//! disables the sandbox boundary check when `true`.

mod error;
mod markdown;
mod ops;
mod tools;

use std::path::PathBuf;

pub use tools::{all_tools, ExtractTocTool, MarkdownStatsTool};

/// Workspace root and sandbox settings for markdown tools (aligned with [`crate::fs::FsContext`]).
#[derive(Debug, Clone)]
pub struct MdContext {
    pub root_canonical: PathBuf,
    pub allow_outside_root: bool,
}

impl MdContext {
    /// `root: None` uses [`std::env::current_dir`].
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
    use std::fs;
    use std::sync::Arc;

    use serde_json::json;

    use super::*;

    fn tmp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "agentool_md_test_{}_{}",
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
    async fn extract_toc_and_markdown_stats() {
        let root = tmp_root();
        let md = "# Title\n\n## Sub\n\nBody text.\n";
        fs::write(root.join("doc.md"), md).unwrap();
        let ctx = Arc::new(MdContext::new(Some(root.clone()), false).unwrap());
        let tools = all_tools(ctx);

        let toc_tool = tools.iter().find(|t| t.name() == "extract_toc").unwrap();
        let out = toc_tool.execute(json!({ "path": "doc.md" })).await.unwrap();
        assert_eq!(out["success"], true);
        let items = out["data"]["toc"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["level"], 1);
        assert_eq!(items[0]["title"], "Title");
        assert_eq!(items[0]["line"], 1);
        assert_eq!(items[1]["level"], 2);
        assert_eq!(items[1]["line"], 3);

        let stats_tool = tools.iter().find(|t| t.name() == "markdown_stats").unwrap();
        let c = stats_tool
            .execute(json!({ "path": "doc.md" }))
            .await
            .unwrap();
        assert_eq!(c["success"], true);
        assert_eq!(c["data"]["headings"], 2);
        assert!(c["data"]["lines"].as_u64().unwrap() >= 4);
        assert!(c["data"]["characters"].as_u64().unwrap() > 0);

        let _ = fs::remove_dir_all(&root);
    }
}
