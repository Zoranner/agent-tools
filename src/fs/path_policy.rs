use std::path::{Component, Path, PathBuf};

use crate::{ToolError, ToolErrorCode};

use super::error_map::{map_io_error, tool_error};

/// Lexical normalization: collapse `.` / `..` without touching the filesystem.
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                out.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::Normal(part) => out.push(part),
        }
    }
    out
}

/// `path` is equal to `root` or a strict descendant (component-wise).
pub(crate) fn is_descendant(root: &Path, path: &Path) -> bool {
    let r: Vec<_> = root.components().collect();
    let p: Vec<_> = path.components().collect();
    if p.len() < r.len() {
        return false;
    }
    r.iter().zip(p.iter()).all(|(a, b)| a == b)
}

pub(crate) fn ensure_under_root(root: &Path, resolved: &Path) -> Result<(), ToolError> {
    if is_descendant(root, resolved) {
        Ok(())
    } else {
        Err(tool_error(
            ToolErrorCode::InvalidPath,
            "path escapes working root (sandbox)",
        ))
    }
}

/// Resolve a user-supplied path against `base` (cwd or root), then lexical-normalize.
pub(crate) fn combine_and_normalize(base: &Path, user: &Path) -> PathBuf {
    let combined = if user.is_absolute() {
        user.to_path_buf()
    } else {
        base.join(user)
    };
    normalize_path(&combined)
}

/// When `logical` does not exist, walk up until an existing ancestor; return
/// `canonical(existing_ancestor).join(trailing relative tail)`.
pub(crate) fn resolve_with_existing_prefix(logical: &Path) -> Result<PathBuf, ToolError> {
    if logical.exists() {
        return logical
            .canonicalize()
            .map_err(|e| map_io_error(e, "canonicalize"));
    }

    let mut cur = logical.to_path_buf();
    let mut tail_parts: Vec<std::ffi::OsString> = Vec::new();

    while !cur.exists() {
        let Some(name) = cur.file_name() else {
            return Err(tool_error(
                ToolErrorCode::InvalidPath,
                "invalid path (no file name component)",
            ));
        };
        tail_parts.push(name.to_os_string());
        if !cur.pop() {
            break;
        }
    }

    let base = if cur.as_os_str().is_empty() {
        std::env::current_dir().map_err(|e| map_io_error(e, "cwd"))?
    } else {
        cur.canonicalize()
            .map_err(|e| map_io_error(e, "canonicalize"))?
    };

    let mut out = base;
    for name in tail_parts.into_iter().rev() {
        out.push(name);
    }
    Ok(out)
}

/// Resolve `logical` (lexically normalized) inside sandbox `root_canonical`.
pub(crate) fn resolve_sandboxed(
    root_canonical: &Path,
    logical: &Path,
) -> Result<PathBuf, ToolError> {
    if logical.exists() {
        let c = logical
            .canonicalize()
            .map_err(|e| map_io_error(e, "canonicalize"))?;
        ensure_under_root(root_canonical, &c)?;
        return Ok(c);
    }

    let mut cur = logical.to_path_buf();
    let mut tail_parts: Vec<std::ffi::OsString> = Vec::new();

    while !cur.exists() {
        let Some(name) = cur.file_name() else {
            return Err(tool_error(
                ToolErrorCode::InvalidPath,
                "invalid path (no file name component)",
            ));
        };
        tail_parts.push(name.to_os_string());
        if !cur.pop() {
            return Err(tool_error(
                ToolErrorCode::FileNotFound,
                "path has no existing parent under sandbox",
            ));
        }
    }

    let base = cur
        .canonicalize()
        .map_err(|e| map_io_error(e, "canonicalize"))?;
    ensure_under_root(root_canonical, &base)?;

    let mut out = base;
    for name in tail_parts.into_iter().rev() {
        out.push(name);
    }
    ensure_under_root(root_canonical, &out)?;
    Ok(out)
}
