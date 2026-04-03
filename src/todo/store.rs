use std::fs;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::error::{tool_error, TodoErrorCode};
use crate::tool::ToolError;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TodoStatus {
    #[default]
    Pending,
    Done,
    Cancelled,
}

impl TodoStatus {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "pending" => Ok(Self::Pending),
            "done" => Ok(Self::Done),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(tool_error(
                TodoErrorCode::InvalidStatus,
                format!("status must be \"pending\", \"done\", or \"cancelled\", got {other:?}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TodoPriority {
    Low,
    Medium,
    High,
}

impl TodoPriority {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            other => Err(tool_error(
                TodoErrorCode::InvalidPriority,
                format!("priority must be \"low\", \"medium\", or \"high\", got {other:?}"),
            )),
        }
    }

    pub(crate) fn sort_rank(self) -> u8 {
        match self {
            Self::High => 0,
            Self::Medium => 1,
            Self::Low => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TodoItem {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: TodoStatus,
    #[serde(default)]
    pub priority: Option<TodoPriority>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct TodoStore {
    #[serde(default)]
    pub items: Vec<TodoItem>,
}

pub(crate) fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub(crate) fn load(path: &Path) -> Result<TodoStore, ToolError> {
    if !path.exists() {
        return Ok(TodoStore::default());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| tool_error(TodoErrorCode::StorageError, format!("read todo store: {e}")))?;
    if raw.trim().is_empty() {
        return Ok(TodoStore::default());
    }
    serde_json::from_str(&raw).map_err(|e| {
        tool_error(
            TodoErrorCode::StorageError,
            format!("invalid todo store JSON: {e}"),
        )
    })
}

pub(crate) fn save(path: &Path, store: &TodoStore) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            tool_error(
                TodoErrorCode::StorageError,
                format!("create todo store directory: {e}"),
            )
        })?;
    }
    let raw = serde_json::to_string_pretty(store).map_err(|e| {
        tool_error(
            TodoErrorCode::StorageError,
            format!("serialize todo store: {e}"),
        )
    })?;
    fs::write(path, raw).map_err(|e| {
        tool_error(
            TodoErrorCode::StorageError,
            format!("write todo store: {e}"),
        )
    })
}

pub(crate) fn status_sort_rank(s: &TodoStatus) -> u8 {
    match s {
        TodoStatus::Pending => 0,
        TodoStatus::Done => 1,
        TodoStatus::Cancelled => 2,
    }
}

pub(crate) fn priority_rank(item: &TodoItem) -> u8 {
    item.priority.map(TodoPriority::sort_rank).unwrap_or(3)
}
