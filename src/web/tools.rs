use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::{Tool, ToolResult};

use super::ops::{op_web_fetch, op_web_search};
use super::WebContext;

macro_rules! web_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<WebContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<WebContext>) -> Self {
                Self { ctx }
            }
        }

        #[async_trait]
        impl Tool for $name {
            fn name(&self) -> &str {
                $tool_name
            }

            fn description(&self) -> &str {
                $desc
            }

            fn schema(&self) -> Value {
                $schema
            }

            async fn execute(&self, params: Value) -> ToolResult {
                $op(&self.ctx, &params).await
            }
        }
    };
}

web_tool!(
    WebSearchTool,
    "web_search",
    "Search the web and return titles, URLs, and snippets (default: DuckDuckGo; replace via WebContext).",
    schema = json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Search query" },
            "limit": {
                "type": "number",
                "description": "Max results (default 5, max 20)"
            }
        },
        "required": ["query"]
    }),
    op = op_web_search
);

web_tool!(
    WebFetchTool,
    "web_fetch",
    "Fetch a web page over HTTP(S) and return Markdown (default: direct fetch + htmd; replace via WebContext).",
    schema = json!({
        "type": "object",
        "properties": {
            "url": { "type": "string", "description": "Page URL (http or https only)" }
        },
        "required": ["url"]
    }),
    op = op_web_fetch
);

/// Network tools (`web_search`, `web_fetch`) sharing the same [`WebContext`] HTTP client.
pub fn all_tools(ctx: Arc<WebContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(WebSearchTool::new(ctx.clone())),
        Arc::new(WebFetchTool::new(ctx)),
    ]
}
