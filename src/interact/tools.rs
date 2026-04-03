use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::{Tool, ToolResult};

use super::ops;
use super::InteractContext;

macro_rules! interact_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<InteractContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<InteractContext>) -> Self {
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
                let ctx = self.ctx.clone();
                ops::$op(&ctx, &params).await
            }
        }
    };
}

interact_tool!(
    InteractAskTool,
    "interact_ask",
    "Ask the user a question and wait for a reply. Requires an InteractBackend in InteractContext (CLI, UI, etc.). Optional `options` turns this into single-choice; `timeout` is in seconds and is enforced by the backend.",
    schema = json!({
        "type": "object",
        "properties": {
            "question": { "type": "string", "description": "Question text" },
            "options": {
                "type": "array",
                "items": { "type": "string" },
                "description": "If non-empty, user must pick one of these"
            },
            "timeout": { "type": "integer", "description": "Optional timeout in seconds" }
        },
        "required": ["question"]
    }),
    op = op_ask
);

interact_tool!(
    InteractConfirmTool,
    "interact_confirm",
    "Ask the user for yes/no confirmation. `default` is used when the backend times out or cannot obtain an answer (default false). `timeout` is in seconds.",
    schema = json!({
        "type": "object",
        "properties": {
            "message": { "type": "string", "description": "Confirmation prompt" },
            "default": { "type": "boolean", "description": "Fallback when no explicit answer (default false)" },
            "timeout": { "type": "integer", "description": "Optional timeout in seconds" }
        },
        "required": ["message"]
    }),
    op = op_confirm
);

interact_tool!(
    InteractNotifyTool,
    "interact_notify",
    "Send a one-way notification to the user (no reply). Returns whether the backend reported successful delivery.",
    schema = json!({
        "type": "object",
        "properties": {
            "message": { "type": "string", "description": "Notification body" },
            "level": {
                "type": "string",
                "enum": ["info", "warning", "error"],
                "description": "Severity (default info)"
            }
        },
        "required": ["message"]
    }),
    op = op_notify
);

pub fn all_tools(ctx: Arc<InteractContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(InteractAskTool::new(ctx.clone())),
        Arc::new(InteractConfirmTool::new(ctx.clone())),
        Arc::new(InteractNotifyTool::new(ctx)),
    ]
}
