use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::{ToolError, ToolErrorCode, ToolResult};

use super::error_map::{map_io_error, tool_error};
use super::path_policy::{combine_and_normalize, resolve_sandboxed, resolve_with_existing_prefix};
use super::FsContext;

fn json_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
        tool_error(
            ToolErrorCode::InvalidPath,
            format!("missing or invalid `{key}`"),
        )
    })
}

fn json_u64_opt(params: &Value, key: &str) -> Result<Option<u64>, ToolError> {
    let Some(x) = params.get(key) else {
        return Ok(None);
    };
    if x.is_null() {
        return Ok(None);
    }
    if let Some(n) = x.as_u64() {
        return Ok(Some(n));
    }
    if let Some(f) = x.as_f64() {
        if f >= 0.0 && f.is_finite() {
            return Ok(Some(f as u64));
        }
    }
    Err(tool_error(
        ToolErrorCode::InvalidPath,
        format!("`{key}` must be a non-negative number"),
    ))
}

fn ok_data(data: Value) -> Value {
    json!({
        "success": true,
        "data": data,
    })
}

fn resolve(ctx: &FsContext, user: &str) -> Result<PathBuf, ToolError> {
    let s = user.trim();
    if s.is_empty() {
        return Err(tool_error(ToolErrorCode::InvalidPath, "path is empty"));
    }
    let user_path = Path::new(s);
    let base = if ctx.allow_outside_root {
        std::env::current_dir().map_err(|e| map_io_error(e, "cwd"))?
    } else {
        ctx.root_canonical.clone()
    };
    let logical = combine_and_normalize(&base, user_path);

    if ctx.allow_outside_root {
        if logical.exists() {
            return logical
                .canonicalize()
                .map_err(|e| map_io_error(e, "canonicalize"));
        }
        return resolve_with_existing_prefix(&logical);
    }

    resolve_sandboxed(&ctx.root_canonical, &logical)
}

pub(crate) fn op_read_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let offset = json_u64_opt(params, "offset")?;
    let limit = json_u64_opt(params, "limit")?;
    if let Some(0) = offset {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "`offset` must be >= 1 when provided",
        ));
    }
    let resolved = resolve(ctx, path)?;
    if !resolved.is_file() {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "path is not a regular file",
        ));
    }
    let text = fs::read_to_string(&resolved).map_err(|e| map_io_error(e, "read_file"))?;
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len() as u64;
    let start = offset
        .map(|o| (o - 1) as usize)
        .unwrap_or(0)
        .min(lines.len());
    let content = if let Some(lim) = limit {
        lines
            .into_iter()
            .skip(start)
            .take(lim as usize)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        lines.into_iter().skip(start).collect::<Vec<_>>().join("\n")
    };
    Ok(ok_data(json!({
        "content": content,
        "total_lines": total_lines,
    })))
}

pub(crate) fn op_write_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let content = json_str(params, "content")?;
    let resolved = resolve(ctx, path)?;
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|e| map_io_error(e, "create_dir_all"))?;
    }
    let mut f = fs::File::create(&resolved).map_err(|e| map_io_error(e, "create"))?;
    f.write_all(content.as_bytes())
        .map_err(|e| map_io_error(e, "write"))?;
    let abs = resolved
        .canonicalize()
        .unwrap_or(resolved)
        .display()
        .to_string();
    Ok(ok_data(json!({ "path": abs })))
}

pub(crate) fn op_edit_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let old_text = json_str(params, "old_text")?;
    let new_text = json_str(params, "new_text")?;
    if old_text.is_empty() {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "`old_text` must not be empty",
        ));
    }
    let resolved = resolve(ctx, path)?;
    if !resolved.is_file() {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "path is not a regular file",
        ));
    }
    let text = fs::read_to_string(&resolved).map_err(|e| map_io_error(e, "read_file"))?;
    let count = text.match_indices(old_text).count();
    match count {
        0 => Err(tool_error(
            ToolErrorCode::PatternNotFound,
            "`old_text` not found in file",
        )),
        1 => {
            let updated = text.replacen(old_text, new_text, 1);
            fs::write(&resolved, updated).map_err(|e| map_io_error(e, "write"))?;
            let abs = resolved
                .canonicalize()
                .unwrap_or(resolved)
                .display()
                .to_string();
            Ok(ok_data(json!({ "path": abs })))
        }
        _ => Err(tool_error(
            ToolErrorCode::PatternNotUnique,
            "`old_text` matches multiple locations",
        )),
    }
}

pub(crate) fn op_create_directory(ctx: &FsContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let resolved = resolve(ctx, path)?;
    fs::create_dir_all(&resolved).map_err(|e| map_io_error(e, "create_dir_all"))?;
    let abs = resolved
        .canonicalize()
        .unwrap_or(resolved)
        .display()
        .to_string();
    Ok(ok_data(json!({ "path": abs })))
}

pub(crate) fn op_list_directory(ctx: &FsContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let resolved = resolve(ctx, path)?;
    if !resolved.is_dir() {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "path is not a directory",
        ));
    }
    let mut entries: Vec<Value> = Vec::new();
    for e in fs::read_dir(&resolved).map_err(|e| map_io_error(e, "read_dir"))? {
        let e = e.map_err(|e| map_io_error(e, "read_dir"))?;
        let meta = e.metadata().map_err(|e| map_io_error(e, "metadata"))?;
        let name = e.file_name().to_string_lossy().to_string();
        let (ty, size) = if meta.is_dir() {
            ("directory", 0_u64)
        } else {
            ("file", meta.len())
        };
        entries.push(json!({
            "name": name,
            "type": ty,
            "size": size,
        }));
    }
    entries.sort_by(|a, b| {
        let na = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        na.cmp(nb)
    });
    Ok(ok_data(json!({ "entries": entries })))
}

pub(crate) fn op_delete_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let resolved = resolve(ctx, path)?;
    let meta = fs::metadata(&resolved).map_err(|e| map_io_error(e, "metadata"))?;
    if meta.is_dir() {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "`delete_file` only removes regular files, not directories",
        ));
    }
    let abs = resolved
        .canonicalize()
        .unwrap_or_else(|_| resolved.clone())
        .display()
        .to_string();
    fs::remove_file(&resolved).map_err(|e| map_io_error(e, "remove_file"))?;
    Ok(ok_data(json!({ "path": abs })))
}

fn ensure_dest_absent(dest: &Path) -> Result<(), ToolError> {
    if dest.exists() {
        return Err(tool_error(
            ToolErrorCode::FileAlreadyExists,
            "destination path already exists",
        ));
    }
    Ok(())
}

pub(crate) fn op_move_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let source = json_str(params, "source")?;
    let destination = json_str(params, "destination")?;
    let src = resolve(ctx, source)?;
    if !src.is_file() {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "`source` is not a regular file",
        ));
    }
    let dst = resolve(ctx, destination)?;
    ensure_dest_absent(&dst)?;
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| map_io_error(e, "create_dir_all"))?;
    }
    if fs::rename(&src, &dst).is_err() {
        fs::copy(&src, &dst).map_err(|e| map_io_error(e, "copy"))?;
        fs::remove_file(&src).map_err(|e| map_io_error(e, "remove_file"))?;
    }
    let src_abs = src.canonicalize().unwrap_or(src).display().to_string();
    let dst_abs = dst.canonicalize().unwrap_or(dst).display().to_string();
    Ok(ok_data(json!({
        "source": src_abs,
        "destination": dst_abs,
    })))
}

pub(crate) fn op_copy_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let source = json_str(params, "source")?;
    let destination = json_str(params, "destination")?;
    let src = resolve(ctx, source)?;
    if !src.is_file() {
        return Err(tool_error(
            ToolErrorCode::InvalidPath,
            "`source` is not a regular file",
        ));
    }
    let dst = resolve(ctx, destination)?;
    ensure_dest_absent(&dst)?;
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| map_io_error(e, "create_dir_all"))?;
    }
    fs::copy(&src, &dst).map_err(|e| map_io_error(e, "copy"))?;
    let src_abs = src.canonicalize().unwrap_or(src).display().to_string();
    let dst_abs = dst.canonicalize().unwrap_or(dst).display().to_string();
    Ok(ok_data(json!({
        "source": src_abs,
        "destination": dst_abs,
    })))
}

/// Run an fs operation in a blocking task (std::fs).
pub(crate) async fn run_blocking<F>(f: F) -> ToolResult
where
    F: FnOnce() -> ToolResult + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(super::error_map::join_blocking_error)?
}
