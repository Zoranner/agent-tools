//! 在阻塞线程池中执行同步工具逻辑（`std::fs`、`git2` 等）。

use crate::tool::ToolResult;

pub(crate) async fn run_blocking<F>(f: F) -> ToolResult
where
    F: FnOnce() -> ToolResult + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(crate::tool::join_blocking_error)?
}
