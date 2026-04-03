use std::path::PathBuf;

use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::json::{json_str, ok_data};
use crate::core::path::resolve_against_workspace_root;
use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, TodoErrorCode};
use super::store::{
    load, now_rfc3339, priority_rank, save, status_sort_rank, TodoItem, TodoPriority, TodoStatus,
    TodoStore,
};
use super::TodoContext;

fn store_path(ctx: &TodoContext) -> Result<PathBuf, ToolError> {
    resolve_against_workspace_root(
        &ctx.root_canonical,
        ctx.allow_outside_root,
        ctx.store_relative.to_str().ok_or_else(|| {
            tool_error(
                TodoErrorCode::StorageError,
                "todo store path is not valid UTF-8",
            )
        })?,
    )
}

fn tags_from_params(params: &Value) -> Vec<String> {
    params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn find_index(store: &TodoStore, id: &str) -> Option<usize> {
    store.items.iter().position(|x| x.id == id)
}

pub(crate) fn op_todo_add(ctx: &TodoContext, params: &Value) -> ToolResult {
    let title = json_str(params, "title")?.trim();
    if title.is_empty() {
        return Err(tool_error(
            TodoErrorCode::InvalidInput,
            "`title` must be non-empty",
        ));
    }
    let description = params
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let priority = match params.get("priority").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => Some(TodoPriority::parse(s)?),
    };
    let tags = tags_from_params(params);

    let path = store_path(ctx)?;
    let mut store: TodoStore = load(&path)?;
    let now = now_rfc3339();
    let id = Uuid::new_v4().to_string();
    store.items.push(TodoItem {
        id: id.clone(),
        title: title.to_string(),
        description,
        status: TodoStatus::Pending,
        priority,
        tags,
        created_at: now.clone(),
        updated_at: now,
    });
    save(&path, &store)?;
    Ok(ok_data(json!({ "id": id })))
}

pub(crate) fn op_todo_list(ctx: &TodoContext, params: &Value) -> ToolResult {
    let status_filter = match params.get("status").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => Some(TodoStatus::parse(s)?),
    };
    let tag_filter = params.get("tag").and_then(|v| v.as_str());
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50)
        .clamp(1, 200) as usize;

    let path = store_path(ctx)?;
    let store: TodoStore = load(&path)?;
    let mut items: Vec<TodoItem> = store
        .items
        .into_iter()
        .filter(|it| {
            if let Some(ref st) = status_filter {
                if &it.status != st {
                    return false;
                }
            }
            if let Some(t) = tag_filter {
                if !it.tags.iter().any(|x| x == t) {
                    return false;
                }
            }
            true
        })
        .collect();

    items.sort_by(|a, b| {
        let s = status_sort_rank(&a.status).cmp(&status_sort_rank(&b.status));
        if s != std::cmp::Ordering::Equal {
            return s;
        }
        let p = priority_rank(a).cmp(&priority_rank(b));
        if p != std::cmp::Ordering::Equal {
            return p;
        }
        b.updated_at.cmp(&a.updated_at)
    });
    items.truncate(limit);

    let list: Vec<Value> = items
        .into_iter()
        .map(|it| serde_json::to_value(&it).expect("TodoItem should always serialize to JSON"))
        .collect();

    Ok(ok_data(json!({ "items": list })))
}

pub(crate) fn op_todo_update(ctx: &TodoContext, params: &Value) -> ToolResult {
    let id = json_str(params, "id")?;
    let path = store_path(ctx)?;
    let mut store: TodoStore = load(&path)?;
    let idx = find_index(&store, id)
        .ok_or_else(|| tool_error(TodoErrorCode::NotFound, format!("no todo with id `{id}`")))?;
    let now = now_rfc3339();
    {
        let it = &mut store.items[idx];
        if let Some(v) = params.get("title") {
            if let Some(s) = v.as_str() {
                let t = s.trim();
                if t.is_empty() {
                    return Err(tool_error(
                        TodoErrorCode::InvalidInput,
                        "`title` cannot be empty when provided",
                    ));
                }
                it.title = t.to_string();
            }
        }
        if let Some(v) = params.get("description") {
            if let Some(s) = v.as_str() {
                it.description = s.to_string();
            }
        }
        if let Some(v) = params.get("status") {
            if let Some(s) = v.as_str() {
                it.status = TodoStatus::parse(s)?;
            }
        }
        if let Some(v) = params.get("priority") {
            if v.is_null() {
                it.priority = None;
            } else if let Some(s) = v.as_str() {
                it.priority = Some(TodoPriority::parse(s)?);
            }
        }
        if let Some(arr) = params.get("tags").and_then(|v| v.as_array()) {
            it.tags = arr
                .iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect();
        }
        it.updated_at = now;
    }
    save(&path, &store)?;
    Ok(ok_data(json!({ "id": id })))
}

pub(crate) fn op_todo_remove(ctx: &TodoContext, params: &Value) -> ToolResult {
    let id = json_str(params, "id")?;
    let path = store_path(ctx)?;
    let mut store: TodoStore = load(&path)?;
    let idx = find_index(&store, id)
        .ok_or_else(|| tool_error(TodoErrorCode::NotFound, format!("no todo with id `{id}`")))?;
    store.items.remove(idx);
    save(&path, &store)?;
    Ok(ok_data(json!({ "id": id, "removed": true })))
}
