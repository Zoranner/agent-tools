use std::path::{Path, PathBuf};

use crate::core::path::resolve_against_workspace_root;
use crate::tool::ToolError;

use super::error::{tool_error, FindErrorCode};

/// Resolve the directory (or single file) to scan under.
pub(crate) fn resolve_find_root(
    workspace_root: &Path,
    allow_outside_root: bool,
    path_param: Option<&str>,
) -> Result<PathBuf, ToolError> {
    let logical = match path_param.map(str::trim).filter(|s| !s.is_empty()) {
        None => workspace_root.to_path_buf(),
        Some(s) => resolve_against_workspace_root(workspace_root, allow_outside_root, s)?,
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
    if root == file {
        return file
            .file_name()
            .map(|name| name.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| file.to_string_lossy().replace('\\', "/"));
    }
    file.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| file.to_string_lossy().replace('\\', "/"))
}
