use std::io::ErrorKind;

use crate::{ToolError, ToolErrorCode};

pub(crate) fn tool_error(code: ToolErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code,
        message: message.into(),
    }
}

pub(crate) fn map_io_error(err: std::io::Error, context: &str) -> ToolError {
    let code = match err.kind() {
        ErrorKind::NotFound => ToolErrorCode::FileNotFound,
        ErrorKind::PermissionDenied => ToolErrorCode::PermissionDenied,
        ErrorKind::AlreadyExists => ToolErrorCode::FileAlreadyExists,
        ErrorKind::DirectoryNotEmpty => ToolErrorCode::DirectoryNotEmpty,
        ErrorKind::InvalidInput | ErrorKind::InvalidData => ToolErrorCode::InvalidPath,
        #[cfg(target_os = "windows")]
        ErrorKind::InvalidFilename => ToolErrorCode::InvalidPath,
        // Avoid misreporting unrelated I/O failures as FILE_NOT_FOUND; message still carries OS text.
        _ => ToolErrorCode::InvalidPath,
    };
    let prefix = if context.is_empty() {
        String::new()
    } else {
        format!("{context}: ")
    };
    ToolError {
        code,
        message: format!("{prefix}{err}"),
    }
}

pub(crate) fn join_blocking_error(err: tokio::task::JoinError) -> ToolError {
    tool_error(
        ToolErrorCode::InvalidPath,
        format!("blocking task failed: {err}"),
    )
}
