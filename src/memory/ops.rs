use std::path::PathBuf;

use chrono::Utc;
use serde_json::{json, Value};

use crate::core::json::{json_str, ok_data};
use crate::core::path::resolve_against_workspace_root;
use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, MemoryErrorCode};
use super::store::{
    append_block, collect_all_blocks, daily_file_path, summary_file_path, ParsedBlock,
};
use super::MemoryContext;

fn memory_root(ctx: &MemoryContext) -> Result<PathBuf, ToolError> {
    resolve_against_workspace_root(
        &ctx.root_canonical,
        ctx.allow_outside_root,
        ctx.memory_dir_relative.to_str().ok_or_else(|| {
            tool_error(
                MemoryErrorCode::StorageError,
                "memory directory path is not valid UTF-8",
            )
        })?,
    )
}

fn normalize_key(key: &str) -> Result<&str, ToolError> {
    let k = key.trim();
    if k.is_empty() {
        return Err(tool_error(
            MemoryErrorCode::InvalidKey,
            "memory key must be non-empty",
        ));
    }
    Ok(k)
}

fn parse_target(params: &Value) -> Result<&'static str, ToolError> {
    match params.get("target").and_then(|v| v.as_str()) {
        None | Some("daily") => Ok("daily"),
        Some("summary") => Ok("summary"),
        Some(other) => Err(tool_error(
            MemoryErrorCode::InvalidTarget,
            format!("`target` must be \"daily\" or \"summary\", got {other:?}"),
        )),
    }
}

fn tags_from_params(params: &Value) -> Vec<String> {
    params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn block_matches_tags(block: &ParsedBlock, required: &[String]) -> bool {
    if required.is_empty() {
        return true;
    }
    required.iter().all(|t| block.tags.iter().any(|et| et == t))
}

fn pick_latest_for_key(blocks: Vec<ParsedBlock>, key: &str) -> Option<ParsedBlock> {
    blocks.into_iter().filter(|b| b.key == key).max_by(|a, b| {
        let ord = a.at.cmp(&b.at);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
        a.file_rel.cmp(&b.file_rel)
    })
}

pub(crate) fn op_memory_write(ctx: &MemoryContext, params: &Value) -> ToolResult {
    let key = normalize_key(json_str(params, "key")?)?;
    let content = json_str(params, "content")?;
    let tags = tags_from_params(params);
    let target = parse_target(params)?;

    let root = memory_root(ctx)?;
    let path = match target {
        "daily" => daily_file_path(&root, Utc::now().date_naive()),
        "summary" => summary_file_path(&root),
        _ => unreachable!(),
    };
    append_block(&path, key, content, &tags)?;

    let rel = path
        .strip_prefix(&root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.file_name().unwrap().to_string_lossy().into_owned());

    Ok(ok_data(json!({
        "key": key,
        "target": target,
        "path": rel,
    })))
}

pub(crate) fn op_memory_read(ctx: &MemoryContext, params: &Value) -> ToolResult {
    let key = normalize_key(json_str(params, "key")?)?;
    let root = memory_root(ctx)?;
    let blocks = collect_all_blocks(&root)?;
    let block = pick_latest_for_key(blocks, key).ok_or_else(|| {
        tool_error(
            MemoryErrorCode::KeyNotFound,
            format!("no memory block for key `{key}`"),
        )
    })?;
    Ok(ok_data(json!({
        "key": key,
        "content": block.content,
        "tags": block.tags,
        "file": block.file_rel,
        "created_at": block.at,
        "updated_at": block.at,
    })))
}

pub(crate) fn op_memory_search(ctx: &MemoryContext, params: &Value) -> ToolResult {
    let query = json_str(params, "query")?.trim();
    let query_lc = query.to_lowercase();
    let filter_tags = tags_from_params(params);

    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .clamp(1, 100) as usize;

    let root = memory_root(ctx)?;
    let mut blocks = collect_all_blocks(&root)?;

    blocks.retain(|b| {
        if !block_matches_tags(b, &filter_tags) {
            return false;
        }
        if query.is_empty() {
            return true;
        }
        b.key.to_lowercase().contains(&query_lc) || b.content.to_lowercase().contains(&query_lc)
    });

    blocks.sort_by(|a, b| {
        let ord = b.at.cmp(&a.at);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
        a.file_rel.cmp(&b.file_rel)
    });
    blocks.truncate(limit);

    let results: Vec<Value> = blocks
        .into_iter()
        .map(|b| {
            json!({
                "key": b.key,
                "content": b.content,
                "tags": b.tags,
                "file": b.file_rel,
            })
        })
        .collect();

    Ok(ok_data(json!({ "results": results })))
}
