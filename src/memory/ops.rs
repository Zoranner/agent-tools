use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::{json, Value};

use crate::core::json::{json_str, json_str_opt, json_string_array_opt, json_u64_opt, ok_data};
use crate::core::path::resolve_against_workspace_root;
use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, MemoryErrorCode};
use super::store::{
    any_key_exists, append_block, block_kind, collect_all_blocks, collect_located_blocks,
    daily_file_path, is_summary_rel, remove_span_and_append_block, summary_file_path, LocatedBlock,
    ParsedBlock,
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
    match json_str_opt(params, "target")? {
        None | Some("daily") => Ok("daily"),
        Some("summary") => Ok("summary"),
        Some(other) => Err(tool_error(
            MemoryErrorCode::InvalidTarget,
            format!("`target` must be \"daily\" or \"summary\", got {other:?}"),
        )),
    }
}

fn tags_from_params(params: &Value) -> Result<Vec<String>, ToolError> {
    Ok(json_string_array_opt(params, "tags")?.unwrap_or_default())
}

fn block_matches_tags(block: &ParsedBlock, required: &[String]) -> bool {
    if required.is_empty() {
        return true;
    }
    required.iter().all(|t| block.tags.iter().any(|et| et == t))
}

/// 无时间戳的块排在最旧，便于 `max_by` 比较。
fn at_sort_key(at: &str) -> &str {
    if at.is_empty() {
        "1970-01-01T00:00:00+00:00"
    } else {
        at
    }
}

fn cmp_located_canonical(a: &LocatedBlock, b: &LocatedBlock) -> Ordering {
    let o = at_sort_key(&a.block.at).cmp(at_sort_key(&b.block.at));
    if o != Ordering::Equal {
        return o;
    }
    a.block.file_rel.cmp(&b.block.file_rel)
}

/// 读取 / 更新时：若存在 `MEMORY.md` 中同名块，只在该集合里取最新；否则在全部日记忆中取最新。
fn pick_canonical_located(located: Vec<LocatedBlock>, key: &str) -> Option<LocatedBlock> {
    let matches: Vec<_> = located.into_iter().filter(|l| l.block.key == key).collect();
    if matches.is_empty() {
        return None;
    }
    let summary_matches: Vec<_> = matches
        .iter()
        .filter(|l| is_summary_rel(&l.block.file_rel))
        .cloned()
        .collect();
    let pool = if summary_matches.is_empty() {
        matches
    } else {
        summary_matches
    };
    pool.into_iter().max_by(cmp_located_canonical)
}

fn rel_path_under_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
}

pub(crate) fn op_memory_write(ctx: &MemoryContext, params: &Value) -> ToolResult {
    let key = normalize_key(json_str(params, "key")?)?;
    let content = json_str(params, "content")?;
    let tags = tags_from_params(params)?;
    let target = parse_target(params)?;

    let root = memory_root(ctx)?;
    let existing = collect_all_blocks(&root)?;
    if any_key_exists(&existing, key) {
        return Err(tool_error(
            MemoryErrorCode::KeyExists,
            format!(
                "memory key `{key}` already exists; call memory_read to inspect, then memory_update to replace the canonical block"
            ),
        ));
    }

    let path = if target == "summary" {
        summary_file_path(&root)
    } else {
        daily_file_path(&root, Utc::now().date_naive())
    };
    append_block(&path, key, content, &tags)?;

    let rel = rel_path_under_root(&root, &path);

    Ok(ok_data(json!({
        "key": key,
        "target": target,
        "path": rel,
    })))
}

pub(crate) fn op_memory_update(ctx: &MemoryContext, params: &Value) -> ToolResult {
    let key = normalize_key(json_str(params, "key")?)?;
    let content = json_str(params, "content")?;
    let tags = tags_from_params(params)?;

    let root = memory_root(ctx)?;
    let located = collect_located_blocks(&root)?;
    let target = pick_canonical_located(located, key).ok_or_else(|| {
        tool_error(
            MemoryErrorCode::KeyNotFound,
            format!("no memory block for key `{key}`; use memory_write to create"),
        )
    })?;

    remove_span_and_append_block(&target.abs_path, target.byte_range, key, content, &tags)?;

    let rel = rel_path_under_root(&root, &target.abs_path);
    Ok(ok_data(json!({
        "key": key,
        "path": rel,
        "kind": block_kind(&target.block.file_rel),
    })))
}

pub(crate) fn op_memory_read(ctx: &MemoryContext, params: &Value) -> ToolResult {
    let key = normalize_key(json_str(params, "key")?)?;
    let root = memory_root(ctx)?;
    let located = collect_located_blocks(&root)?;
    let loc = pick_canonical_located(located, key).ok_or_else(|| {
        tool_error(
            MemoryErrorCode::KeyNotFound,
            format!("no memory block for key `{key}`"),
        )
    })?;
    let b = loc.block;
    let kind = block_kind(&b.file_rel);
    Ok(ok_data(json!({
        "key": key,
        "content": b.content,
        "tags": b.tags,
        "file": b.file_rel,
        "kind": kind,
        "created_at": b.at,
        "updated_at": b.at,
    })))
}

pub(crate) fn op_memory_search(ctx: &MemoryContext, params: &Value) -> ToolResult {
    let query = json_str(params, "query")?.trim();
    let query_lc = query.to_lowercase();
    let filter_tags = tags_from_params(params)?;

    let limit = json_u64_opt(params, "limit")?.unwrap_or(10).clamp(1, 100) as usize;

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
        let sa = is_summary_rel(&a.file_rel);
        let sb = is_summary_rel(&b.file_rel);
        let kind_ord = sb.cmp(&sa);
        if kind_ord != Ordering::Equal {
            return kind_ord;
        }
        let o = at_sort_key(&b.at).cmp(at_sort_key(&a.at));
        if o != Ordering::Equal {
            return o;
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
                "kind": block_kind(&b.file_rel),
            })
        })
        .collect();

    Ok(ok_data(json!({ "results": results })))
}
