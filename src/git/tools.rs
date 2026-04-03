use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::tool::{Tool, ToolResult};

use super::ops::{self, run_blocking};
use super::GitContext;

macro_rules! git_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<GitContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<GitContext>) -> Self {
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

git_tool!(
    GitStatusTool,
    "git_status",
    "List working tree changes (added, modified, deleted, untracked) for a Git repository. Optional `path` is resolved against the context default root then discovered as a repo.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Repository path (default: context root)" }
        }
    }),
    op = op_git_status
);

git_tool!(
    GitDiffTool,
    "git_diff",
    "Show unified diff for unstaged changes (default) or staged changes (`staged: true`).",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Repository path (default: context root)" },
            "staged": { "type": "boolean", "description": "If true, diff index vs HEAD; else working tree vs index (default false)" }
        }
    }),
    op = op_git_diff
);

git_tool!(
    GitCommitTool,
    "git_commit",
    "Stage and commit: either all changes under the repo (`files` omitted) or only listed paths. Uses `user.name` / `user.email` from repo config, falling back to agentool defaults.",
    schema = json!({
        "type": "object",
        "properties": {
            "message": { "type": "string", "description": "Commit message" },
            "path": { "type": "string", "description": "Repository path (default: context root)" },
            "files": {
                "type": "array",
                "items": { "type": "string" },
                "description": "If set, stage only these paths (non-empty); if omitted, stage all (`git add .`)"
            }
        },
        "required": ["message"]
    }),
    op = op_git_commit
);

git_tool!(
    GitLogTool,
    "git_log",
    "List recent commits (newest first), up to `limit` (default 10, max 100).",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Repository path (default: context root)" },
            "limit": { "type": "integer", "description": "Max commits (default 10, max 100)" }
        }
    }),
    op = op_git_log
);

pub fn all_tools(ctx: Arc<GitContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(GitStatusTool::new(ctx.clone())),
        Arc::new(GitDiffTool::new(ctx.clone())),
        Arc::new(GitCommitTool::new(ctx.clone())),
        Arc::new(GitLogTool::new(ctx)),
    ]
}
