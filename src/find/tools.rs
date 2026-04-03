use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::blocking::run_blocking;
use crate::{Tool, ToolResult};

use super::ops;
use super::FindContext;

macro_rules! find_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<FindContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<FindContext>) -> Self {
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

find_tool!(
    GrepSearchTool,
    "grep_search",
    "Search file contents with a regular expression under a directory (or single file).",
    schema = json!({
        "type": "object",
        "properties": {
            "pattern": { "type": "string", "description": "Regular expression pattern" },
            "path": { "type": "string", "description": "Root directory or file (default: context root)" },
            "glob": { "type": "string", "description": "Optional glob filter for paths relative to root, e.g. **/*.md" },
            "ignore_case": { "type": "boolean", "description": "Case-insensitive matching (default false)" }
        },
        "required": ["pattern"]
    }),
    op = op_grep_search
);

find_tool!(
    GlobSearchTool,
    "glob_search",
    "List files under a directory whose relative path matches a glob pattern.",
    schema = json!({
        "type": "object",
        "properties": {
            "pattern": { "type": "string", "description": "Glob pattern, e.g. **/*.md" },
            "path": { "type": "string", "description": "Root directory (default: context root)" }
        },
        "required": ["pattern"]
    }),
    op = op_glob_search
);

/// Local find tools (`grep_search`, `glob_search`) sharing the same [`FindContext`].
pub fn all_tools(ctx: Arc<FindContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(GrepSearchTool::new(ctx.clone())),
        Arc::new(GlobSearchTool::new(ctx)),
    ]
}
