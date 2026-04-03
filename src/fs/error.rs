use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum FsErrorCode {
    InvalidPath,
    PatternNotFound,
    PatternNotUnique,
    FileAlreadyExists,
}

impl FsErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPath => "INVALID_PATH",
            Self::PatternNotFound => "PATTERN_NOT_FOUND",
            Self::PatternNotUnique => "PATTERN_NOT_UNIQUE",
            Self::FileAlreadyExists => "FILE_ALREADY_EXISTS",
        }
    }
}

pub(crate) fn tool_error(code: FsErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
