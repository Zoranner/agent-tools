//! Long-lived memory tools: [`memory_write`](MemoryWriteTool), [`memory_update`](MemoryUpdateTool),
//! [`memory_read`](MemoryReadTool), [`memory_search`](MemorySearchTool).
//!
//! Default layout (OpenClaw-style, paths adjusted): **`.agent/memory/YYYY/MM/dd.md`** for append-only daily notes,
//! and **`.agent/memory/MEMORY.md`** for curated long-term summary. See [`MemoryContext`].

mod error;
mod ops;
mod store;
mod tools;

use std::path::{Path, PathBuf};

pub use tools::{all_tools, MemoryReadTool, MemorySearchTool, MemoryUpdateTool, MemoryWriteTool};

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
    async fn write_read_search_update_and_summary_priority() {
        let root = tmp_root();
        let ctx = Arc::new(
            MemoryContext::with_memory_dir_relative(Some(root.clone()), false, Path::new("mem"))
                .unwrap(),
        );
        let tools = all_tools(ctx);

        let write = tools.iter().find(|t| t.name() == "memory_write").unwrap();
        let update = tools.iter().find(|t| t.name() == "memory_update").unwrap();
        let read = tools.iter().find(|t| t.name() == "memory_read").unwrap();
        let search = tools.iter().find(|t| t.name() == "memory_search").unwrap();

        write
            .execute(json!({
                "key": "k1",
                "content": "hello 中文 world",
                "tags": ["a", "doc"],
                "target": "daily"
            }))
            .await
            .unwrap();

        let dup = write
            .execute(json!({ "key": "k1", "content": "nope", "target": "daily" }))
            .await
            .unwrap_err();
        assert_eq!(dup.code, "MEMORY_KEY_EXISTS");

        let out = read.execute(json!({ "key": "k1" })).await.unwrap();
        assert_eq!(out["data"]["content"], "hello 中文 world");
        assert_eq!(out["data"]["kind"], "daily");
        let tags = out["data"]["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 2);

        update
            .execute(json!({ "key": "k1", "content": "updated body" }))
            .await
            .unwrap();
        let out2 = read.execute(json!({ "key": "k1" })).await.unwrap();
        assert_eq!(out2["data"]["content"], "updated body");

        let s = search
            .execute(json!({ "query": "updated", "limit": 5 }))
            .await
            .unwrap();
        assert_eq!(s["data"]["results"].as_array().unwrap().len(), 1);

        let mem_root = root.join("mem");
        let summary = mem_root.join("MEMORY.md");

        write
            .execute(json!({
                "key": "dup_key",
                "content": "daily dup",
                "target": "daily"
            }))
            .await
            .unwrap();
        fs::write(
            &summary,
            r####"### dup_key

summary wins

<!-- agentool-memory: at=2099-01-01T00:00:00+00:00 tags=s -->

"####,
        )
        .unwrap();
        let dup_read = read.execute(json!({ "key": "dup_key" })).await.unwrap();
        assert_eq!(dup_read["data"]["kind"], "summary");
        assert!(dup_read["data"]["content"]
            .as_str()
            .unwrap()
            .contains("summary wins"));

        write
            .execute(json!({
                "key": "sort_a",
                "content": "needle",
                "target": "daily"
            }))
            .await
            .unwrap();
        write
            .execute(json!({
                "key": "sort_b",
                "content": "needle summary",
                "target": "summary"
            }))
            .await
            .unwrap();
        let needle = search
            .execute(json!({ "query": "needle", "limit": 10 }))
            .await
            .unwrap();
        let arr = needle["data"]["results"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["kind"], "summary");
        assert_eq!(arr[1]["kind"], "daily");

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

    #[tokio::test]
    async fn rejects_invalid_optional_param_types() {
        let root = tmp_root();
        let ctx = Arc::new(
            MemoryContext::with_memory_dir_relative(Some(root.clone()), false, Path::new("mem"))
                .unwrap(),
        );
        let tools = all_tools(ctx);

        let write = tools.iter().find(|t| t.name() == "memory_write").unwrap();
        let err = write
            .execute(json!({
                "key": "k1",
                "content": "body",
                "tags": "ops"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let err = write
            .execute(json!({
                "key": "k1",
                "content": "body",
                "target": true
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let search = tools.iter().find(|t| t.name() == "memory_search").unwrap();
        let err = search
            .execute(json!({
                "query": "",
                "limit": "5"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let _ = fs::remove_dir_all(&root);
    }
}
