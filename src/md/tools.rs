use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::blocking::run_blocking;
use crate::{Tool, ToolResult};

use super::ops;
use super::MdContext;

macro_rules! md_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<MdContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<MdContext>) -> Self {
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

md_tool!(
    ExtractTocTool,
    "toc_extract",
    "Extract ATX heading outline (# .. ######) from a Markdown file. Headings inside fenced code blocks are ignored. Paths resolve against the context root; sandbox mode keeps reads inside the workspace.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Path to the .md file (relative to workspace root or absolute in relaxed mode)" }
        },
        "required": ["path"]
    }),
    op = op_extract_toc
);

md_tool!(
    MarkdownStatsTool,
    "markdown_inspect",
    "Non-whitespace character count, paragraph count (blank-line separated), ATX heading count, and line count for a Markdown file. Fenced code blocks are excluded from character and paragraph counts. Paths resolve like other workspace tools.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Path to the Markdown file" }
        },
        "required": ["path"]
    }),
    op = op_markdown_stats
);

pub fn all_tools(ctx: Arc<MdContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ExtractTocTool::new(ctx.clone())),
        Arc::new(MarkdownStatsTool::new(ctx)),
    ]
}
