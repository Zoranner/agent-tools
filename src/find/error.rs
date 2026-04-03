use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum FindErrorCode {
    InvalidPath,
    InvalidPattern,
    FileNotFound,
    PermissionDenied,
}

impl FindErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPath => "INVALID_PATH",
            Self::InvalidPattern => "INVALID_PATTERN",
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::PermissionDenied => "PERMISSION_DENIED",
        }
    }
}

pub(crate) fn tool_error(code: FindErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
