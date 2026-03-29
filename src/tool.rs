use async_trait::async_trait;
use serde_json::Value;

use crate::ToolResult;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, params: Value) -> ToolResult;
}
