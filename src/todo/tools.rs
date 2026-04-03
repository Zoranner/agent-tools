use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::blocking::run_blocking;
use crate::{Tool, ToolResult};

use super::ops;
use super::TodoContext;

macro_rules! todo_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<TodoContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<TodoContext>) -> Self {
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
                run_blocking(move || ops::$op(&ctx, &params)).await
            }
        }
    };
}

todo_tool!(
    TodoAddTool,
    "todo_add",
    "Create a todo item (UUID id) in the workspace `.agent/todos.json` store. Default status is pending.",
    schema = json!({
        "type": "object",
        "properties": {
            "title": { "type": "string", "description": "Short title (required, non-empty)" },
            "description": { "type": "string", "description": "Optional longer text" },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high"],
                "description": "Optional priority"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional labels; todo_list can filter by tag"
            }
        },
        "required": ["title"]
    }),
    op = op_todo_add
);

todo_tool!(
    TodoListTool,
    "todo_list",
    "List todos from the JSON store: pending first, then higher priority, then most recently updated. Optional status/tag filter.",
    schema = json!({
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["pending", "done", "cancelled"],
                "description": "If set, only this status"
            },
            "tag": { "type": "string", "description": "If set, item must include this tag" },
            "limit": { "type": "integer", "description": "Max items (default 50, max 200)" }
        }
    }),
    op = op_todo_list
);

todo_tool!(
    TodoUpdateTool,
    "todo_update",
    "Patch fields on an existing todo by id. Only provided fields change. Set `priority` to JSON null to clear priority. `tags` replaces the whole list when provided.",
    schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Todo id from todo_add" },
            "title": { "type": "string" },
            "description": { "type": "string" },
            "status": {
                "type": "string",
                "enum": ["pending", "done", "cancelled"]
            },
            "priority": {
                "description": "Optional: \"low\", \"medium\", or \"high\"; JSON null clears priority"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["id"]
    }),
    op = op_todo_update
);

todo_tool!(
    TodoRemoveTool,
    "todo_remove",
    "Delete a todo by id from the store.",
    schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Todo id to remove" }
        },
        "required": ["id"]
    }),
    op = op_todo_remove
);

pub fn all_tools(ctx: Arc<TodoContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(TodoAddTool::new(ctx.clone())),
        Arc::new(TodoListTool::new(ctx.clone())),
        Arc::new(TodoUpdateTool::new(ctx.clone())),
        Arc::new(TodoRemoveTool::new(ctx)),
    ]
}
