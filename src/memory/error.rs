use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum MemoryErrorCode {
    KeyNotFound,
    InvalidKey,
    InvalidTarget,
    StorageError,
}

impl MemoryErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::KeyNotFound => "MEMORY_KEY_NOT_FOUND",
            Self::InvalidKey => "MEMORY_INVALID_KEY",
            Self::InvalidTarget => "MEMORY_INVALID_TARGET",
            Self::StorageError => "MEMORY_STORAGE_ERROR",
        }
    }
}

pub(crate) fn tool_error(code: MemoryErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
