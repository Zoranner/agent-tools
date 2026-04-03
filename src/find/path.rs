use std::path::{Component, Path, PathBuf};

use crate::tool::ToolError;

use super::error::{tool_error, FindErrorCode};

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

pub(crate) fn combine_and_normalize(base: &Path, user: &Path) -> PathBuf {
    let combined = if user.is_absolute() {
        user.to_path_buf()
    } else {
        base.join(user)
    };
    normalize_path(&combined)
}

/// Resolve the directory (or single file) to scan under.
pub(crate) fn resolve_find_root(
    workspace_root: &Path,
    path_param: Option<&str>,
) -> Result<PathBuf, ToolError> {
    let logical = match path_param.map(str::trim).filter(|s| !s.is_empty()) {
        None => workspace_root.to_path_buf(),
        Some(s) => combine_and_normalize(workspace_root, Path::new(s)),
    };
    if !logical.exists() {
        return Err(tool_error(
            FindErrorCode::FileNotFound,
            "find root path does not exist",
        ));
    }
    logical.canonicalize().map_err(|e| {
        let code = match e.kind() {
            std::io::ErrorKind::NotFound => FindErrorCode::FileNotFound,
            std::io::ErrorKind::PermissionDenied => FindErrorCode::PermissionDenied,
            _ => FindErrorCode::InvalidPath,
        };
        tool_error(code, format!("resolve find root: {e}"))
    })
}

pub(crate) fn display_path_relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| file.to_string_lossy().replace('\\', "/"))
}
