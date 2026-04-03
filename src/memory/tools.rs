use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::blocking::run_blocking;
use crate::{Tool, ToolResult};

use super::ops;
use super::MemoryContext;

macro_rules! memory_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<MemoryContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<MemoryContext>) -> Self {
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

memory_tool!(
    MemoryWriteTool,
    "memory_write",
    "Create a new memory block only if `key` does not exist anywhere under the memory directory. `target: daily` writes to YYYY/MM/dd.md (UTC date); `target: summary` appends to MEMORY.md. On duplicate key, returns MEMORY_KEY_EXISTS — use memory_read then memory_update.",
    schema = json!({
        "type": "object",
        "properties": {
            "key": { "type": "string", "description": "Single-line heading key (### key); must be globally unique" },
            "content": { "type": "string", "description": "Body text under the heading" },
            "target": {
                "type": "string",
                "enum": ["daily", "summary"],
                "description": "daily = dated log; summary = MEMORY.md"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional labels in footer metadata"
            }
        },
        "required": ["key", "content"]
    }),
    op = op_memory_write
);

memory_tool!(
    MemoryUpdateTool,
    "memory_update",
    "Replace the canonical block for `key`: same resolution as memory_read (summary in MEMORY.md wins over daily logs; then newest `at`). Removes that block from disk and appends the updated block at the end of the same file.",
    schema = json!({
        "type": "object",
        "properties": {
            "key": { "type": "string", "description": "Existing ### key" },
            "content": { "type": "string", "description": "New body text" },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional labels (replaces footer tags)"
            }
        },
        "required": ["key", "content"]
    }),
    op = op_memory_update
);

memory_tool!(
    MemoryReadTool,
    "memory_read",
    "Load the canonical block for `key`: if any block exists in MEMORY.md, the newest among those is returned; otherwise the newest among daily logs. Response includes `kind`: \"summary\" or \"daily\".",
    schema = json!({
        "type": "object",
        "properties": {
            "key": { "type": "string", "description": "The ### heading key" }
        },
        "required": ["key"]
    }),
    op = op_memory_read
);

memory_tool!(
    MemorySearchTool,
    "memory_search",
    "Substring search (case-insensitive) over keys and bodies. Results include `kind` (summary vs daily). Summary entries are listed before daily; within each tier, newer `at` first.",
    schema = json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Substring; empty matches any (subject to tag filter)" },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Block must contain all listed tags"
            },
            "limit": { "type": "integer", "description": "Max results (default 10, max 100)" }
        },
        "required": ["query"]
    }),
    op = op_memory_search
);

pub fn all_tools(ctx: Arc<MemoryContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(MemoryWriteTool::new(ctx.clone())),
        Arc::new(MemoryUpdateTool::new(ctx.clone())),
        Arc::new(MemoryReadTool::new(ctx.clone())),
        Arc::new(MemorySearchTool::new(ctx)),
    ]
}
