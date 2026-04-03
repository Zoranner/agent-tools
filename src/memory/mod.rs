//! Long-lived memory tools: [`memory_write`](MemoryWriteTool), [`memory_read`](MemoryReadTool), [`memory_search`](MemorySearchTool).
//!
//! Default layout (OpenClaw-style, paths adjusted): **`.agent/memory/YYYY/MM/dd.md`** for append-only daily notes,
//! and **`.agent/memory/MEMORY.md`** for curated long-term summary. See [`MemoryContext`].

mod error;
mod ops;
mod store;
mod tools;

use std::path::{Path, PathBuf};

pub use tools::{all_tools, MemoryReadTool, MemorySearchTool, MemoryWriteTool};

/// Workspace root, optional sandbox bypass, and relative path to the memory **directory** (not a single file).
#[derive(Debug, Clone)]
pub struct MemoryContext {
    pub root_canonical: PathBuf,
    pub allow_outside_root: bool,
    /// Directory under `root_canonical`, e.g. `.agent/memory`.
    pub memory_dir_relative: PathBuf,
}

impl MemoryContext {
    /// `root: None` uses [`std::env::current_dir`]. Memory directory defaults to `.agent/memory`.
    pub fn new(root: Option<PathBuf>, allow_outside_root: bool) -> std::io::Result<Self> {
        Self::with_memory_dir_relative(root, allow_outside_root, Path::new(".agent/memory"))
    }

    /// Same as [`Self::new`] but with a custom directory relative to the workspace root.
    pub fn with_memory_dir_relative(
        root: Option<PathBuf>,
        allow_outside_root: bool,
        memory_dir_relative: &Path,
    ) -> std::io::Result<Self> {
        let r = match root {
            Some(p) => p,
            None => std::env::current_dir()?,
        };
        Ok(Self {
            root_canonical: r.canonicalize()?,
            allow_outside_root,
            memory_dir_relative: memory_dir_relative.to_path_buf(),
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
            "agentool_memory_test_{}_{}",
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
    async fn write_read_search_daily_and_summary() {
        let root = tmp_root();
        let ctx = Arc::new(
            MemoryContext::with_memory_dir_relative(Some(root.clone()), false, Path::new("mem"))
                .unwrap(),
        );
        let tools = all_tools(ctx);

        let write = tools.iter().find(|t| t.name() == "memory_write").unwrap();
        write
            .execute(json!({
                "key": "k1",
                "content": "hello 中文 world",
                "tags": ["a", "doc"],
                "target": "daily"
            }))
            .await
            .unwrap();

        let read = tools.iter().find(|t| t.name() == "memory_read").unwrap();
        let out = read.execute(json!({ "key": "k1" })).await.unwrap();
        assert_eq!(out["data"]["content"], "hello 中文 world");
        let tags = out["data"]["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 2);
        let file = out["data"]["file"].as_str().unwrap();
        assert!(file.ends_with(".md"));
        assert!(file.contains('/'));

        let mem_root = root.join("mem");
        let daily_path = mem_root.join(file);
        assert!(daily_path.is_file());
        let raw = fs::read_to_string(&daily_path).unwrap();
        assert!(raw.contains("### k1"));
        assert!(raw.contains("hello 中文 world"));

        let search = tools.iter().find(|t| t.name() == "memory_search").unwrap();
        let s = search
            .execute(json!({ "query": "中文", "limit": 5 }))
            .await
            .unwrap();
        let results = s["data"]["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["key"], "k1");

        let filtered = search
            .execute(json!({ "query": "", "tags": ["a"], "limit": 10 }))
            .await
            .unwrap();
        assert_eq!(filtered["data"]["results"].as_array().unwrap().len(), 1);

        write
            .execute(json!({
                "key": "pref",
                "content": "long-term fact",
                "target": "summary"
            }))
            .await
            .unwrap();
        let summary = mem_root.join("MEMORY.md");
        assert!(summary.is_file());
        let sum_raw = fs::read_to_string(&summary).unwrap();
        assert!(sum_raw.contains("### pref"));
        assert!(sum_raw.contains("long-term fact"));

        let s2 = search
            .execute(json!({ "query": "long-term", "limit": 5 }))
            .await
            .unwrap();
        assert_eq!(s2["data"]["results"].as_array().unwrap().len(), 1);

        let miss = read.execute(json!({ "key": "missing" })).await.unwrap_err();
        assert_eq!(miss.code, "MEMORY_KEY_NOT_FOUND");

        let bad = write
            .execute(json!({
                "key": "x",
                "content": "y",
                "target": "nope"
            }))
            .await
            .unwrap_err();
        assert_eq!(bad.code, "MEMORY_INVALID_TARGET");

        let _ = fs::remove_dir_all(&root);
    }
}
