use crate::tool::ToolError;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TaskErrorCode {
    InvalidInput,
    InvalidStatus,
    InvalidKind,
    InvalidOwner,
    InvalidPriority,
    NotFound,
    LockConflict,
    StorageError,
}

impl TaskErrorCode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "TASK_INVALID_INPUT",
            Self::InvalidStatus => "TASK_INVALID_STATUS",
            Self::InvalidKind => "TASK_INVALID_KIND",
            Self::InvalidOwner => "TASK_INVALID_OWNER",
            Self::InvalidPriority => "TASK_INVALID_PRIORITY",
            Self::NotFound => "TASK_NOT_FOUND",
            Self::LockConflict => "TASK_LOCK_CONFLICT",
            Self::StorageError => "TASK_STORAGE_ERROR",
        }
    }
}

pub(crate) fn task_error(code: TaskErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
