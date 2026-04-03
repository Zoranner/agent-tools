use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

/// Unified JSON error shape: `error.code` / `error.message`. Stable codes are defined per tool module (`fs::error`, `web::error`, …).
#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct ToolError {
    pub code: String,
    pub message: String,
}

pub type ToolResult = Result<Value, ToolError>;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, params: Value) -> ToolResult;
}
