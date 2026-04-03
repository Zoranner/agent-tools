use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum TodoErrorCode {
    NotFound,
    InvalidInput,
    InvalidStatus,
    InvalidPriority,
    StorageError,
}

impl TodoErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "TODO_NOT_FOUND",
            Self::InvalidInput => "TODO_INVALID_INPUT",
            Self::InvalidStatus => "TODO_INVALID_STATUS",
            Self::InvalidPriority => "TODO_INVALID_PRIORITY",
            Self::StorageError => "TODO_STORAGE_ERROR",
        }
    }
}

pub(crate) fn tool_error(code: TodoErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
