use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::blocking::run_blocking;
use crate::tool::{Tool, ToolResult};

use super::ops;
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

git_tool!(
    WorktreeAddTool,
    "worktree_add",
    "Create a new linked worktree. If `branch` does not exist it is created from HEAD.",
    schema = json!({
        "type": "object",
        "properties": {
            "name":   { "type": "string", "description": "Worktree name" },
            "path":   { "type": "string", "description": "Directory path for the new worktree" },
            "branch": { "type": "string", "description": "Branch to check out; created from HEAD if absent" },
            "repo":   { "type": "string", "description": "Repository path (default: context root)" }
        },
        "required": ["name", "path"]
    }),
    op = op_worktree_add
);

git_tool!(
    WorktreeListTool,
    "worktree_list",
    "List all linked worktrees with their path and lock status.",
    schema = json!({
        "type": "object",
        "properties": {
            "repo": { "type": "string", "description": "Repository path (default: context root)" }
        }
    }),
    op = op_worktree_list
);

git_tool!(
    WorktreeRemoveTool,
    "worktree_remove",
    "Remove a linked worktree. Use `force: true` to remove even if the worktree is locked.",
    schema = json!({
        "type": "object",
        "properties": {
            "name":  { "type": "string", "description": "Worktree name" },
            "force": { "type": "boolean", "description": "Force removal even if the worktree is locked (default false)" },
            "repo":  { "type": "string", "description": "Repository path (default: context root)" }
        },
        "required": ["name"]
    }),
    op = op_worktree_remove
);

git_tool!(
    WorktreeLockTool,
    "worktree_lock",
    "Lock a worktree to prevent accidental removal.",
    schema = json!({
        "type": "object",
        "properties": {
            "name":   { "type": "string", "description": "Worktree name" },
            "reason": { "type": "string", "description": "Optional lock reason" },
            "repo":   { "type": "string", "description": "Repository path (default: context root)" }
        },
        "required": ["name"]
    }),
    op = op_worktree_lock
);

git_tool!(
    WorktreeUnlockTool,
    "worktree_unlock",
    "Unlock a previously locked worktree.",
    schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Worktree name" },
            "repo": { "type": "string", "description": "Repository path (default: context root)" }
        },
        "required": ["name"]
    }),
    op = op_worktree_unlock
);

pub fn all_tools(ctx: Arc<GitContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(GitStatusTool::new(ctx.clone())),
        Arc::new(GitDiffTool::new(ctx.clone())),
        Arc::new(GitCommitTool::new(ctx.clone())),
        Arc::new(GitLogTool::new(ctx.clone())),
        Arc::new(WorktreeAddTool::new(ctx.clone())),
        Arc::new(WorktreeListTool::new(ctx.clone())),
        Arc::new(WorktreeRemoveTool::new(ctx.clone())),
        Arc::new(WorktreeLockTool::new(ctx.clone())),
        Arc::new(WorktreeUnlockTool::new(ctx.clone())),
    ]
}
