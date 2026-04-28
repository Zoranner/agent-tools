use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::RegexBuilder;
use serde_json::{json, Value};
use walkdir::WalkDir;

use crate::core::json::{json_bool_opt, json_str, json_str_opt, json_u64_opt, ok_data};
use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, FindErrorCode};

use super::path::{display_path_relative, resolve_find_root};
use super::FindContext;

const DEFAULT_RESULT_LIMIT: usize = 100;
const MAX_RESULT_LIMIT: usize = 1_000;

fn trimmed_str_opt<'a>(params: &'a Value, key: &str) -> Result<Option<&'a str>, ToolError> {
    Ok(json_str_opt(params, key)?
        .map(str::trim)
        .filter(|s| !s.is_empty()))
}

fn result_limit(params: &Value) -> Result<usize, ToolError> {
    let limit = json_u64_opt(params, "limit")?.unwrap_or(DEFAULT_RESULT_LIMIT as u64);
    if limit == 0 {
        return Err(tool_error(
            FindErrorCode::InvalidPath,
            "`limit` must be at least 1",
        ));
    }
    Ok((limit as usize).min(MAX_RESULT_LIMIT))
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
    if root == file {
        let candidate = file
            .file_name()
            .map(|name| name.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        return matcher.is_match(&candidate);
    }
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

    let root = resolve_find_root(
        &ctx.root_canonical,
        ctx.allow_outside_root,
        trimmed_str_opt(params, "path")?,
    )?;
    let ignore_case = json_bool_opt(params, "ignore_case")?.unwrap_or(false);
    let limit = result_limit(params)?;

    let re = RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
        .map_err(|e| {
            tool_error(
                FindErrorCode::InvalidPattern,
                format!("invalid regular expression: {e}"),
            )
        })?;

    let glob_matcher = match trimmed_str_opt(params, "glob")? {
        Some(g) => Some(build_glob_set(g)?),
        None => None,
    };

    let mut matches = Vec::new();
    let mut truncated = false;

    let walker = WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| e.file_name() != OsStr::new(".git"));

    'entries: for entry in walker {
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
                if matches.len() >= limit {
                    truncated = true;
                    break 'entries;
                }
                matches.push(json!({
                    "file": display_path_relative(&root, p),
                    "line": (i + 1) as u64,
                    "content": line,
                }));
            }
        }
    }

    Ok(ok_data(json!({
        "matches": matches,
        "truncated": truncated,
    })))
}

pub(crate) fn op_glob_search(ctx: &FindContext, params: &Value) -> ToolResult {
    let pattern = json_str(params, "pattern")?;
    if pattern.trim().is_empty() {
        return Err(tool_error(
            FindErrorCode::InvalidPath,
            "`pattern` must not be empty",
        ));
    }

    let root = resolve_find_root(
        &ctx.root_canonical,
        ctx.allow_outside_root,
        trimmed_str_opt(params, "path")?,
    )?;
    let matcher = build_glob_set(pattern)?;
    let limit = result_limit(params)?;

    let mut files = Vec::<PathBuf>::new();
    let mut truncated = false;
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
            let display = display_path_relative(&root, p);
            if files.len() < limit {
                files.push(PathBuf::from(display));
                continue;
            }

            truncated = true;
            let largest = files
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.cmp(b))
                .map(|(idx, path)| (idx, path.clone()));

            if let Some((idx, current_largest)) = largest {
                let candidate = PathBuf::from(display);
                if candidate < current_largest {
                    files[idx] = candidate;
                }
            }
        }
    }

    files.sort();
    let files: Vec<Value> = files
        .into_iter()
        .map(|path| json!(path.to_string_lossy().replace('\\', "/")))
        .collect();

    Ok(ok_data(json!({
        "files": files,
        "truncated": truncated,
    })))
}
