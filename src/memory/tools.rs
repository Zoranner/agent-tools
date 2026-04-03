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
    "Append a Markdown memory block under the workspace `.agent/memory` tree (OpenClaw-inspired). `target: daily` writes to YYYY/MM/dd.md; `target: summary` appends to MEMORY.md. Each block uses `### key` plus machine-readable footer comments for search/read.",
    schema = json!({
        "type": "object",
        "properties": {
            "key": { "type": "string", "description": "Single-line heading key for this block (### key)" },
            "content": { "type": "string", "description": "Body text under the heading" },
            "target": {
                "type": "string",
                "enum": ["daily", "summary"],
                "description": "daily = time-based log file; summary = long-term MEMORY.md"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional labels; stored in footer metadata for memory_search filtering"
            }
        },
        "required": ["key", "content"]
    }),
    op = op_memory_write
);

memory_tool!(
    MemoryReadTool,
    "memory_read",
    "Load the latest memory block with the given key across MEMORY.md and all dated logs under the memory directory.",
    schema = json!({
        "type": "object",
        "properties": {
            "key": { "type": "string", "description": "The ### heading key to find" }
        },
        "required": ["key"]
    }),
    op = op_memory_read
);

memory_tool!(
    MemorySearchTool,
    "memory_search",
    "Substring search (case-insensitive) over keys and bodies in all .md files under the memory directory. Optional tags require every listed tag in the block metadata. Newest blocks (by footer timestamp) first.",
    schema = json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Substring to match; empty matches any (subject to tag filter)" },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "If set, block must contain all of these tags"
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
        Arc::new(MemoryReadTool::new(ctx.clone())),
        Arc::new(MemorySearchTool::new(ctx)),
    ]
}
