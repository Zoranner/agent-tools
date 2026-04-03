//! 在阻塞线程池中执行同步工具逻辑（`std::fs`、`git2` 等）。

use crate::tool::{ToolError, ToolResult};

fn join_blocking_error(err: tokio::task::JoinError) -> ToolError {
    ToolError {
        code: "INVALID_PATH".into(),
        message: format!("blocking task failed: {err}"),
    }
}

pub(crate) async fn run_blocking<F>(f: F) -> ToolResult
where
    F: FnOnce() -> ToolResult + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(join_blocking_error)?
}
