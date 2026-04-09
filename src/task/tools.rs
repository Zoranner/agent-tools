use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::blocking::run_blocking;
use crate::{Tool, ToolResult};

use super::ops;
use super::TaskContext;

macro_rules! task_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<TaskContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<TaskContext>) -> Self {
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
                run_blocking(move || ops::$op(&ctx.db, &params)).await
            }
        }
    };
}

task_tool!(
    TaskCreateTool,
    "task_create",
    "Create a task in the SQLite task store. Returns the new task id. Default status is backlog, default owner is agent.",
    schema = json!({
        "type": "object",
        "properties": {
            "title": { "type": "string", "description": "Short title (required, non-empty)" },
            "description": { "type": "string", "description": "Optional longer description" },
            "kind": {
                "type": "string",
                "enum": ["task", "milestone", "checkpoint"],
                "description": "Task kind (default: task)"
            },
            "owner": {
                "type": "string",
                "enum": ["agent", "human"],
                "description": "Who owns this task (default: agent)"
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high"],
                "description": "Optional priority"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional labels"
            },
            "goal": { "type": "string", "description": "Optional task goal description" },
            "acceptance": { "type": "string", "description": "Optional acceptance criteria" },
            "risk_level": {
                "type": "string",
                "enum": ["low", "medium", "high", "critical"],
                "description": "Optional risk level"
            }
        },
        "required": ["title"]
    }),
    op = op_task_create
);

task_tool!(
    TaskListTool,
    "task_list",
    "List tasks from the store. Supports filtering by status, kind, owner, and tag. Returns up to `limit` items ordered by updated_at desc.",
    schema = json!({
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["backlog", "ready", "running", "waiting_checkpoint", "blocked", "done", "failed", "cancelled"],
                "description": "Filter by status"
            },
            "kind": {
                "type": "string",
                "enum": ["task", "milestone", "checkpoint"],
                "description": "Filter by kind"
            },
            "owner": {
                "type": "string",
                "enum": ["agent", "human"],
                "description": "Filter by owner"
            },
            "tag": { "type": "string", "description": "Filter by tag" },
            "limit": { "type": "integer", "description": "Max items (default 50, max 200)" }
        }
    }),
    op = op_task_list
);

task_tool!(
    TaskGetTool,
    "task_get",
    "Get a single task by id.",
    schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Task id" }
        },
        "required": ["id"]
    }),
    op = op_task_get
);

task_tool!(
    TaskUpdateTool,
    "task_update",
    "Patch fields on an existing task by id. Only provided fields change. Set `priority` or `risk_level` to JSON null to clear them.",
    schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Task id" },
            "title": { "type": "string" },
            "description": { "type": "string" },
            "status": {
                "type": "string",
                "enum": ["backlog", "ready", "running", "waiting_checkpoint", "blocked", "done", "failed", "cancelled"]
            },
            "priority": { "description": "\"low\", \"medium\", \"high\", or JSON null to clear" },
            "blocked_reason": { "description": "String or JSON null to clear" },
            "last_error": { "description": "String or JSON null to clear" },
            "tags": { "type": "array", "items": { "type": "string" } },
            "risk_level": { "description": "\"low\", \"medium\", \"high\", \"critical\", or JSON null to clear" }
        },
        "required": ["id"]
    }),
    op = op_task_update
);

task_tool!(
    TaskDeleteTool,
    "task_delete",
    "Delete a task by id. Cascades to steps, runs, locks, checkpoints, and artifacts.",
    schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Task id to delete" }
        },
        "required": ["id"]
    }),
    op = op_task_delete
);

task_tool!(
    TaskStartRunTool,
    "task_start_run",
    "Start a new execution run for a task. Sets task status to running. Returns run_id.",
    schema = json!({
        "type": "object",
        "properties": {
            "task_id": { "type": "string", "description": "Task id" },
            "step_id": { "type": "string", "description": "Optional step id this run is executing" }
        },
        "required": ["task_id"]
    }),
    op = op_task_start_run
);

task_tool!(
    TaskEndRunTool,
    "task_end_run",
    "End an execution run. Updates run and task status accordingly.",
    schema = json!({
        "type": "object",
        "properties": {
            "run_id": { "type": "string", "description": "Run id from task_start_run" },
            "status": {
                "type": "string",
                "enum": ["done", "failed", "cancelled"],
                "description": "Final run status (default: done)"
            },
            "error": { "type": "string", "description": "Optional error message" },
            "summary": { "type": "string", "description": "Optional execution summary" }
        },
        "required": ["run_id"]
    }),
    op = op_task_end_run
);

task_tool!(
    TaskAppendStepTool,
    "task_append_step",
    "Append a new step to a task. Steps are ordered by seq (auto-incremented).",
    schema = json!({
        "type": "object",
        "properties": {
            "task_id": { "type": "string", "description": "Task id" },
            "title": { "type": "string", "description": "Step title (required, non-empty)" }
        },
        "required": ["task_id", "title"]
    }),
    op = op_task_append_step
);

task_tool!(
    TaskUpdateStepTool,
    "task_update_step",
    "Update the status of a task step.",
    schema = json!({
        "type": "object",
        "properties": {
            "step_id": { "type": "string", "description": "Step id from task_append_step" },
            "status": {
                "type": "string",
                "enum": ["pending", "running", "done", "failed", "cancelled"]
            }
        },
        "required": ["step_id", "status"]
    }),
    op = op_task_update_step
);

task_tool!(
    TaskOpenCheckpointTool,
    "task_open_checkpoint",
    "Open a checkpoint for a task. Sets task status to waiting_checkpoint. Returns checkpoint_id.",
    schema = json!({
        "type": "object",
        "properties": {
            "task_id": { "type": "string", "description": "Task id" },
            "message": { "type": "string", "description": "Checkpoint message or question for the user" },
            "run_id": { "type": "string", "description": "Optional run id" },
            "risk_level": {
                "type": "string",
                "enum": ["low", "medium", "high", "critical"],
                "description": "Optional risk level"
            }
        },
        "required": ["task_id", "message"]
    }),
    op = op_task_open_checkpoint
);

task_tool!(
    TaskCloseCheckpointTool,
    "task_close_checkpoint",
    "Close a checkpoint and resume the task. `task_status` controls what status the task transitions to (default: ready).",
    schema = json!({
        "type": "object",
        "properties": {
            "checkpoint_id": { "type": "string", "description": "Checkpoint id from task_open_checkpoint" },
            "status": {
                "type": "string",
                "enum": ["resolved", "closed"],
                "description": "Checkpoint final status (default: closed)"
            },
            "task_status": {
                "type": "string",
                "enum": ["backlog", "ready", "running", "blocked", "cancelled"],
                "description": "Task status after closing checkpoint (default: ready)"
            }
        },
        "required": ["checkpoint_id"]
    }),
    op = op_task_close_checkpoint
);

task_tool!(
    TaskAcquireLockTool,
    "task_acquire_lock",
    "Acquire an exclusive write lock on a file path for a task. Fails with TASK_LOCK_CONFLICT if already locked.",
    schema = json!({
        "type": "object",
        "properties": {
            "task_id": { "type": "string", "description": "Task id acquiring the lock" },
            "path": { "type": "string", "description": "Canonical file path to lock" },
            "run_id": { "type": "string", "description": "Optional run id" },
            "expires_at": { "type": "string", "description": "Optional RFC3339 expiry time" }
        },
        "required": ["task_id", "path"]
    }),
    op = op_task_acquire_lock
);

task_tool!(
    TaskReleaseLockTool,
    "task_release_lock",
    "Release a previously acquired path lock by lock_id.",
    schema = json!({
        "type": "object",
        "properties": {
            "lock_id": { "type": "string", "description": "Lock id from task_acquire_lock" }
        },
        "required": ["lock_id"]
    }),
    op = op_task_release_lock
);

task_tool!(
    TaskAddArtifactTool,
    "task_add_artifact",
    "Record a task artifact (file, summary, report, or reference). Returns artifact_id.",
    schema = json!({
        "type": "object",
        "properties": {
            "task_id": { "type": "string", "description": "Task id" },
            "kind": { "type": "string", "description": "Artifact kind: file | summary | report | reference (open string)" },
            "run_id": { "type": "string", "description": "Optional run id" },
            "path": { "type": "string", "description": "Optional file path" },
            "content": { "type": "string", "description": "Optional inline content" }
        },
        "required": ["task_id", "kind"]
    }),
    op = op_task_add_artifact
);

pub fn all_tools(ctx: Arc<TaskContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(TaskCreateTool::new(ctx.clone())),
        Arc::new(TaskListTool::new(ctx.clone())),
        Arc::new(TaskGetTool::new(ctx.clone())),
        Arc::new(TaskUpdateTool::new(ctx.clone())),
        Arc::new(TaskDeleteTool::new(ctx.clone())),
        Arc::new(TaskStartRunTool::new(ctx.clone())),
        Arc::new(TaskEndRunTool::new(ctx.clone())),
        Arc::new(TaskAppendStepTool::new(ctx.clone())),
        Arc::new(TaskUpdateStepTool::new(ctx.clone())),
        Arc::new(TaskOpenCheckpointTool::new(ctx.clone())),
        Arc::new(TaskCloseCheckpointTool::new(ctx.clone())),
        Arc::new(TaskAcquireLockTool::new(ctx.clone())),
        Arc::new(TaskReleaseLockTool::new(ctx.clone())),
        Arc::new(TaskAddArtifactTool::new(ctx)),
    ]
}
