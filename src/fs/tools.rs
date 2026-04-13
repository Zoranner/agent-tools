use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::blocking::run_blocking;
use crate::{Tool, ToolResult};

use super::ops;
use super::FsContext;

macro_rules! fs_tool {
    (
        $name:ident, $tool_name:literal, $desc:literal,
        schema = $schema:expr,
        op = $op:ident
    ) => {
        pub struct $name {
            ctx: Arc<FsContext>,
        }

        impl $name {
            pub fn new(ctx: Arc<FsContext>) -> Self {
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

fs_tool!(
    ReadFileTool,
    "file_read",
    "Read a text file with optional line offset and limit.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "File path" },
            "offset": {
                "type": "integer",
                "minimum": 1,
                "description": "Starting line (1-based, integer only)"
            },
            "limit": {
                "type": "integer",
                "minimum": 0,
                "description": "Max lines to read (integer only)"
            }
        },
        "required": ["path"]
    }),
    op = op_read_file
);

fs_tool!(
    WriteFileTool,
    "file_write",
    "Write a file, creating parent directories if needed.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "File path" },
            "content": { "type": "string", "description": "File content" }
        },
        "required": ["path", "content"]
    }),
    op = op_write_file
);

fs_tool!(
    EditFileTool,
    "file_edit",
    "Replace exactly one unique occurrence of old_text with new_text in a file.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "File path" },
            "old_text": { "type": "string", "description": "Text to replace (must be unique)" },
            "new_text": { "type": "string", "description": "Replacement text" }
        },
        "required": ["path", "old_text", "new_text"]
    }),
    op = op_edit_file
);

fs_tool!(
    CreateDirectoryTool,
    "directory_create",
    "Create a directory recursively.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Directory path" }
        },
        "required": ["path"]
    }),
    op = op_create_directory
);

fs_tool!(
    ListDirectoryTool,
    "directory_list",
    "List entries in a directory.",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Directory path" }
        },
        "required": ["path"]
    }),
    op = op_list_directory
);

fs_tool!(
    DeleteFileTool,
    "file_delete",
    "Delete a regular file (not a directory).",
    schema = json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "File path" }
        },
        "required": ["path"]
    }),
    op = op_delete_file
);

fs_tool!(
    MoveFileTool,
    "file_move",
    "Move or rename a file, creating destination parent directories if needed.",
    schema = json!({
        "type": "object",
        "properties": {
            "source": { "type": "string", "description": "Source file path" },
            "destination": { "type": "string", "description": "Destination path" }
        },
        "required": ["source", "destination"]
    }),
    op = op_move_file
);

fs_tool!(
    CopyFileTool,
    "file_copy",
    "Copy a file to a new path, creating destination parent directories if needed.",
    schema = json!({
        "type": "object",
        "properties": {
            "source": { "type": "string", "description": "Source file path" },
            "destination": { "type": "string", "description": "Destination path" }
        },
        "required": ["source", "destination"]
    }),
    op = op_copy_file
);

/// All fs tools sharing the same [`FsContext`].
pub fn all_tools(ctx: Arc<FsContext>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ReadFileTool::new(ctx.clone())),
        Arc::new(WriteFileTool::new(ctx.clone())),
        Arc::new(EditFileTool::new(ctx.clone())),
        Arc::new(CreateDirectoryTool::new(ctx.clone())),
        Arc::new(ListDirectoryTool::new(ctx.clone())),
        Arc::new(DeleteFileTool::new(ctx.clone())),
        Arc::new(MoveFileTool::new(ctx.clone())),
        Arc::new(CopyFileTool::new(ctx)),
    ]
}
