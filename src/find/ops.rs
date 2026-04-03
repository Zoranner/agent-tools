use std::ffi::OsStr;
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::RegexBuilder;
use serde_json::{json, Value};
use walkdir::WalkDir;

use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, FindErrorCode};

use super::path::{display_path_relative, resolve_find_root};
use super::FindContext;

fn ok_data(data: Value) -> Value {
    json!({
        "success": true,
        "data": data,
    })
}

fn json_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
        tool_error(
            FindErrorCode::InvalidPath,
            format!("missing or invalid `{key}`"),
        )
    })
}

fn json_str_opt<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn json_bool_opt(params: &Value, key: &str, default: bool) -> bool {
    params.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

fn build_glob_set(pattern: &str) -> Result<GlobSet, ToolError> {
    let g = Glob::new(pattern).map_err(|e| {
        tool_error(
            FindErrorCode::InvalidPattern,
            format!("invalid glob pattern: {e}"),
        )
    })?;
    GlobSetBuilder::new().add(g).build().map_err(|e| {
        tool_error(
            FindErrorCode::InvalidPattern,
            format!("invalid glob pattern: {e}"),
        )
    })
}

fn path_matches_glob(root: &Path, file: &Path, matcher: &GlobSet) -> bool {
    let rel = match file.strip_prefix(root) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let s = rel.to_string_lossy().replace('\\', "/");
    matcher.is_match(&*s)
}

pub(crate) fn op_grep_search(ctx: &FindContext, params: &Value) -> ToolResult {
    let pattern = json_str(params, "pattern")?;
    if pattern.trim().is_empty() {
        return Err(tool_error(
            FindErrorCode::InvalidPath,
            "`pattern` must not be empty",
        ));
    }

    let root = resolve_find_root(&ctx.root_canonical, json_str_opt(params, "path"))?;
    let ignore_case = json_bool_opt(params, "ignore_case", false);

    let re = RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
        .map_err(|e| {
            tool_error(
                FindErrorCode::InvalidPattern,
                format!("invalid regular expression: {e}"),
            )
        })?;

    let glob_matcher = match json_str_opt(params, "glob") {
        Some(g) => Some(build_glob_set(g)?),
        None => None,
    };

    let mut matches = Vec::new();

    let walker = WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| e.file_name() != OsStr::new(".git"));

    for entry in walker {
        let entry = entry
            .map_err(|e| tool_error(FindErrorCode::InvalidPath, format!("walk directory: {e}")))?;
        let p = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        if let Some(ref gs) = glob_matcher {
            if !path_matches_glob(&root, p, gs) {
                continue;
            }
        }
        let text = match std::fs::read_to_string(p) {
            Ok(t) => t,
            Err(_) => continue,
        };
        for (i, line) in text.lines().enumerate() {
            let line = line.trim_end_matches('\r');
            if re.is_match(line) {
                matches.push(json!({
                    "file": display_path_relative(&root, p),
                    "line": (i + 1) as u64,
                    "content": line,
                }));
            }
        }
    }

    Ok(ok_data(json!({ "matches": matches })))
}

pub(crate) fn op_glob_search(ctx: &FindContext, params: &Value) -> ToolResult {
    let pattern = json_str(params, "pattern")?;
    if pattern.trim().is_empty() {
        return Err(tool_error(
            FindErrorCode::InvalidPath,
            "`pattern` must not be empty",
        ));
    }

    let root = resolve_find_root(&ctx.root_canonical, json_str_opt(params, "path"))?;
    let matcher = build_glob_set(pattern)?;

    let mut files = Vec::new();
    let walker = WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| e.file_name() != OsStr::new(".git"));

    for entry in walker {
        let entry = entry
            .map_err(|e| tool_error(FindErrorCode::InvalidPath, format!("walk directory: {e}")))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if path_matches_glob(&root, p, &matcher) {
            files.push(json!(display_path_relative(&root, p)));
        }
    }

    files.sort_by(|a: &Value, b: &Value| {
        let sa = a.as_str().unwrap_or("");
        let sb = b.as_str().unwrap_or("");
        sa.cmp(sb)
    });

    Ok(ok_data(json!({ "files": files })))
}

/// Run find work on the blocking pool (walkdir + file reads).
pub(crate) async fn run_blocking<F>(f: F) -> ToolResult
where
    F: FnOnce() -> ToolResult + Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| {
        tool_error(
            FindErrorCode::InvalidPath,
            format!("blocking task failed: {e}"),
        )
    })?
}
