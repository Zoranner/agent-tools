use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::core::path::{map_io_error, resolve_against_workspace_root};
use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, FsErrorCode};

use super::FsContext;

fn json_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
        tool_error(
            FsErrorCode::InvalidPath,
            format!("missing or invalid `{key}`"),
        )
    })
}

/// Optional non-negative integer only: JSON must be an integer (`u64` / non-negative `i64`), not a float.
fn json_u64_integer_opt(params: &Value, key: &str) -> Result<Option<u64>, ToolError> {
    let Some(x) = params.get(key) else {
        return Ok(None);
    };
    if x.is_null() {
        return Ok(None);
    }
    if let Some(n) = x.as_u64() {
        return Ok(Some(n));
    }
    if let Some(n) = x.as_i64() {
        if n >= 0 {
            return Ok(Some(n as u64));
        }
    }
    Err(tool_error(
        FsErrorCode::InvalidPath,
        format!("`{key}` must be a non-negative JSON integer"),
    ))
}

fn ok_data(data: Value) -> Value {
    json!({
        "success": true,
        "data": data,
    })
}

fn resolve(ctx: &FsContext, user: &str) -> Result<PathBuf, ToolError> {
    resolve_against_workspace_root(&ctx.root_canonical, ctx.allow_outside_root, user)
}

pub(crate) fn op_read_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let path = json_str(params, "path")?;
    let offset = json_u64_integer_opt(params, "offset")?;
    let limit = json_u64_integer_opt(params, "limit")?;
    if let Some(0) = offset {
        return Err(tool_error(
            FsErrorCode::InvalidPath,
            "`offset` must be >= 1 when provided",
        ));
    }
    let resolved = resolve(ctx, path)?;
    if !resolved.is_file() {
        return Err(tool_error(
            FsErrorCode::InvalidPath,
            "path is not a regular file",
        ));
    }
    let text = fs::read_to_string(&resolved).map_err(|e| map_io_error(e, "read_file"))?;
    let total_lines = text.lines().count() as u64;
    // Split on '\n' rather than `.lines()` so that '\r' in CRLF content is
    // kept on each segment and the trailing empty segment after a final '\n'
    // is preserved; recombining with join("\n") reconstructs the original text.
    let segments: Vec<&str> = text.split('\n').collect();
    let start = offset
        .map(|o| (o - 1) as usize)
        .unwrap_or(0)
        .min(segments.len());
    let content = if let Some(lim) = limit {
        segments
            .into_iter()
            .skip(start)
            .take(lim as usize)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        segments
            .into_iter()
            .skip(start)
            .collect::<Vec<_>>()
            .join("\n")
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
            FsErrorCode::InvalidPath,
            "`old_text` must not be empty",
        ));
    }
    let resolved = resolve(ctx, path)?;
    if !resolved.is_file() {
        return Err(tool_error(
            FsErrorCode::InvalidPath,
            "path is not a regular file",
        ));
    }
    let text = fs::read_to_string(&resolved).map_err(|e| map_io_error(e, "read_file"))?;
    let count = text.match_indices(old_text).count();
    match count {
        0 => Err(tool_error(
            FsErrorCode::PatternNotFound,
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
            FsErrorCode::PatternNotUnique,
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
            FsErrorCode::InvalidPath,
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
            FsErrorCode::InvalidPath,
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
            FsErrorCode::FileAlreadyExists,
            "destination path already exists",
        ));
    }
    Ok(())
}

/// Core rename-or-copy+remove logic, with an injectable rename function for testability.
///
/// If `rename_fn` fails (e.g. cross-device), falls back to `fs::copy` + `fs::remove_file`.
/// If the removal of the source fails after a successful copy, the destination copy is
/// removed on a best-effort basis so as not to leave a stale file behind.
pub(crate) fn move_with_rename_fallback(
    src: &Path,
    dst: &Path,
    rename_fn: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> Result<(), ToolError> {
    if rename_fn(src, dst).is_err() {
        fs::copy(src, dst).map_err(|e| map_io_error(e, "copy"))?;
        if let Err(e) = fs::remove_file(src) {
            let _ = fs::remove_file(dst);
            return Err(map_io_error(e, "remove_file"));
        }
    }
    Ok(())
}

pub(crate) fn op_move_file(ctx: &FsContext, params: &Value) -> ToolResult {
    let source = json_str(params, "source")?;
    let destination = json_str(params, "destination")?;
    let src = resolve(ctx, source)?;
    if !src.is_file() {
        return Err(tool_error(
            FsErrorCode::InvalidPath,
            "`source` is not a regular file",
        ));
    }
    let dst = resolve(ctx, destination)?;
    ensure_dest_absent(&dst)?;
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| map_io_error(e, "create_dir_all"))?;
    }
    move_with_rename_fallback(&src, &dst, |s, d| fs::rename(s, d))?;
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
            FsErrorCode::InvalidPath,
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
        .map_err(crate::tool::join_blocking_error)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "agentool_ops_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).expect("create tmp dir");
        dir
    }

    #[test]
    fn move_fallback_used_when_rename_fails() {
        let dir = tmp_dir();
        let src = dir.join("src.txt");
        let dst = dir.join("dst.txt");
        fs::write(&src, b"fallback-content").unwrap();

        move_with_rename_fallback(&src, &dst, |_, _| {
            Err(std::io::Error::other("forced rename failure"))
        })
        .unwrap();

        assert!(
            !src.exists(),
            "source should be removed after fallback copy+remove"
        );
        assert_eq!(fs::read_to_string(&dst).unwrap(), "fallback-content");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn move_fallback_copy_error_propagates() {
        let dir = tmp_dir();
        // source does not exist — copy will fail after rename is rejected
        let src = dir.join("nonexistent.txt");
        let dst = dir.join("dst.txt");

        let err =
            move_with_rename_fallback(&src, &dst, |_, _| Err(std::io::Error::other("forced")))
                .unwrap_err();

        assert_eq!(err.code, "FILE_NOT_FOUND");
        let _ = fs::remove_dir_all(&dir);
    }
}
