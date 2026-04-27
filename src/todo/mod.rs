//! Workspace todo list: [`todo_add`](TodoAddTool), [`todo_list`](TodoListTool),
//! [`todo_update`](TodoUpdateTool), [`todo_remove`](TodoRemoveTool).
//!
//! Data file defaults to **`.agent/todos.json`** under the workspace root (see [`TodoContext`]).

mod error;
mod ops;
mod store;
mod tools;

use std::path::{Path, PathBuf};

pub use tools::{all_tools, TodoAddTool, TodoListTool, TodoRemoveTool, TodoUpdateTool};

/// Workspace root, sandbox settings, and relative path to the JSON todo store file.
#[derive(Debug, Clone)]
pub struct TodoContext {
    pub root_canonical: PathBuf,
    pub allow_outside_root: bool,
    /// Relative to `root_canonical`, e.g. `.agent/todos.json`.
    pub store_relative: PathBuf,
}

impl TodoContext {
    /// `root: None` uses [`std::env::current_dir`]. Store defaults to `.agent/todos.json`.
    pub fn new(root: Option<PathBuf>, allow_outside_root: bool) -> std::io::Result<Self> {
        Self::with_store_relative(root, allow_outside_root, Path::new(".agent/todos.json"))
    }

    pub fn with_store_relative(
        root: Option<PathBuf>,
        allow_outside_root: bool,
        store_relative: &Path,
    ) -> std::io::Result<Self> {
        let r = match root {
            Some(p) => p,
            None => std::env::current_dir()?,
        };
        Ok(Self {
            root_canonical: r.canonicalize()?,
            allow_outside_root,
            store_relative: store_relative.to_path_buf(),
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
            "agentool_todo_test_{}_{}",
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
    async fn add_list_update_remove() {
        let root = tmp_root();
        let ctx = Arc::new(
            TodoContext::with_store_relative(Some(root.clone()), false, Path::new("todos.json"))
                .unwrap(),
        );
        let tools = all_tools(ctx);

        let add = tools.iter().find(|t| t.name() == "todo_add").unwrap();
        let out = add
            .execute(json!({
                "title": "  Ship feature ",
                "description": "details",
                "priority": "high",
                "tags": ["a", "b"]
            }))
            .await
            .unwrap();
        let id = out["data"]["id"].as_str().unwrap();

        let list = tools.iter().find(|t| t.name() == "todo_list").unwrap();
        let full = list.execute(json!({})).await.unwrap();
        assert_eq!(full["data"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(full["data"]["items"][0]["title"], "Ship feature");

        let filtered = list
            .execute(json!({ "tag": "a", "status": "pending" }))
            .await
            .unwrap();
        assert_eq!(filtered["data"]["items"].as_array().unwrap().len(), 1);

        let update = tools.iter().find(|t| t.name() == "todo_update").unwrap();
        update
            .execute(json!({
                "id": id,
                "status": "done",
                "priority": serde_json::Value::Null
            }))
            .await
            .unwrap();

        let pending = list.execute(json!({ "status": "pending" })).await.unwrap();
        assert_eq!(pending["data"]["items"].as_array().unwrap().len(), 0);

        let remove = tools.iter().find(|t| t.name() == "todo_remove").unwrap();
        remove.execute(json!({ "id": id })).await.unwrap();
        let empty = list.execute(json!({})).await.unwrap();
        assert_eq!(empty["data"]["items"].as_array().unwrap().len(), 0);

        let miss = remove.execute(json!({ "id": "nope" })).await.unwrap_err();
        assert_eq!(miss.code, "TODO_NOT_FOUND");

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn rejects_invalid_optional_param_types() {
        let root = tmp_root();
        let ctx = Arc::new(
            TodoContext::with_store_relative(Some(root.clone()), false, Path::new("todos.json"))
                .unwrap(),
        );
        let tools = all_tools(ctx);

        let add = tools.iter().find(|t| t.name() == "todo_add").unwrap();
        let err = add
            .execute(json!({
                "title": "Ship feature",
                "tags": "ops"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let list = tools.iter().find(|t| t.name() == "todo_list").unwrap();
        let err = list
            .execute(json!({
                "limit": "10"
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let created = add
            .execute(json!({
                "title": "T1"
            }))
            .await
            .unwrap();
        let id = created["data"]["id"].as_str().unwrap();

        let update = tools.iter().find(|t| t.name() == "todo_update").unwrap();
        let err = update
            .execute(json!({
                "id": id,
                "status": true
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");

        let _ = fs::remove_dir_all(&root);
    }
}
