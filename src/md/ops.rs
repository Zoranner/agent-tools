use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::core::json::{json_str, ok_data};
use crate::core::path::{map_io_error, resolve_against_workspace_root};
use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, MdErrorCode};

use super::markdown::{document_stats, extract_toc};
use super::MdContext;

fn resolve_path(ctx: &MdContext, user: &str) -> Result<PathBuf, ToolError> {
    resolve_against_workspace_root(&ctx.root_canonical, ctx.allow_outside_root, user)
}

pub(crate) fn op_extract_toc(ctx: &MdContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let resolved = resolve_path(ctx, path)?;
    if !resolved.is_file() {
        return Err(tool_error(
            MdErrorCode::InvalidPath,
            "path is not a regular file",
        ));
    }
    let text = fs::read_to_string(&resolved).map_err(|e| map_io_error(e, "read markdown file"))?;
    let toc = extract_toc(&text);
    let path_display = resolved.display().to_string();
    Ok(ok_data(json!({
        "path": path_display,
        "toc": toc,
    })))
}

pub(crate) fn op_markdown_stats(ctx: &MdContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let resolved = resolve_path(ctx, path)?;
    if !resolved.is_file() {
        return Err(tool_error(
            MdErrorCode::InvalidPath,
            "path is not a regular file",
        ));
    }
    let text = fs::read_to_string(&resolved).map_err(|e| map_io_error(e, "read markdown file"))?;
    let stats = document_stats(&text);
    let path_display = resolved.display().to_string();
    Ok(ok_data(json!({
        "path": path_display,
        "characters": stats.characters,
        "paragraphs": stats.paragraphs,
        "headings": stats.headings,
        "lines": stats.lines,
    })))
}
