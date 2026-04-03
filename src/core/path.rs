//! 工作区路径：相对根目录拼接、词法规范化、沙箱内解析，以及常见 `std::io::Error` → [`ToolError`](crate::tool::ToolError) 映射。
//!
//! 供 `fs` / `md` 等需要「工作区根 + 可选越界」语义的工具使用；`git` 仅需 [`combine_and_normalize`] 时也可复用。

use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::tool::ToolError;

#[derive(Clone, Copy)]
enum PathResolutionCode {
    InvalidPath,
    FileNotFound,
}

impl PathResolutionCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPath => "INVALID_PATH",
            Self::FileNotFound => "FILE_NOT_FOUND",
        }
    }
}

fn path_tool_error(code: PathResolutionCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}

/// 将常见 `std::io::Error` 映射为稳定 `error.code`（文件类操作通用）。
pub(crate) fn map_io_error(err: std::io::Error, context: &str) -> ToolError {
    let code = match err.kind() {
        ErrorKind::NotFound => "FILE_NOT_FOUND",
        ErrorKind::PermissionDenied => "PERMISSION_DENIED",
        ErrorKind::AlreadyExists => "FILE_ALREADY_EXISTS",
        ErrorKind::DirectoryNotEmpty => "DIRECTORY_NOT_EMPTY",
        ErrorKind::InvalidInput | ErrorKind::InvalidData => "INVALID_PATH",
        #[cfg(target_os = "windows")]
        ErrorKind::InvalidFilename => "INVALID_PATH",
        _ => "INVALID_PATH",
    };
    let prefix = if context.is_empty() {
        String::new()
    } else {
        format!("{context}: ")
    };
    ToolError {
        code: code.to_string(),
        message: format!("{prefix}{err}"),
    }
}

/// 词法规范化：折叠 `.` / `..`，不访问文件系统。
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

/// `path` 等于 `root` 或为其严格子路径（按路径分量比较）。
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
        Err(path_tool_error(
            PathResolutionCode::InvalidPath,
            "path escapes working root (sandbox)",
        ))
    }
}

/// 将用户路径接到 `base` 上再做词法规范化。
pub(crate) fn combine_and_normalize(base: &Path, user: &Path) -> PathBuf {
    let combined = if user.is_absolute() {
        user.to_path_buf()
    } else {
        base.join(user)
    };
    normalize_path(&combined)
}

/// `logical` 不存在时向上找到已存在祖先，再 `canonicalize(祖先) + 尾部相对段`。
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
            return Err(path_tool_error(
                PathResolutionCode::InvalidPath,
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

/// 在沙箱根 `root_canonical` 内解析已词法规范化的 `logical`。
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
            return Err(path_tool_error(
                PathResolutionCode::InvalidPath,
                "invalid path (no file name component)",
            ));
        };
        tail_parts.push(name.to_os_string());
        if !cur.pop() {
            return Err(path_tool_error(
                PathResolutionCode::FileNotFound,
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

/// 与 `fs` 工具一致：相对路径相对 `root_canonical`；`allow_outside_root == true` 时不校验留在根下。
pub(crate) fn resolve_against_workspace_root(
    root_canonical: &Path,
    allow_outside_root: bool,
    user: &str,
) -> Result<PathBuf, ToolError> {
    let s = user.trim();
    if s.is_empty() {
        return Err(path_tool_error(
            PathResolutionCode::InvalidPath,
            "path is empty",
        ));
    }
    let user_path = Path::new(s);
    let logical = combine_and_normalize(root_canonical, user_path);

    if allow_outside_root {
        if logical.exists() {
            return logical
                .canonicalize()
                .map_err(|e| map_io_error(e, "canonicalize"));
        }
        return resolve_with_existing_prefix(&logical);
    }

    resolve_sandboxed(root_canonical, &logical)
}
