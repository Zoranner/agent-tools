use thiserror::Error;

#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct ToolError {
    pub code: ToolErrorCode,
    pub message: String,
}

#[derive(Debug)]
pub enum ToolErrorCode {
    FileNotFound,
    PermissionDenied,
    FileAlreadyExists,
    DirectoryNotEmpty,
    PatternNotUnique,
    PatternNotFound,
    InvalidPath,
    NetworkError,
    GitError,
    MemoryKeyNotFound,
}

impl std::fmt::Display for ToolErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::FileAlreadyExists => "FILE_ALREADY_EXISTS",
            Self::DirectoryNotEmpty => "DIRECTORY_NOT_EMPTY",
            Self::PatternNotUnique => "PATTERN_NOT_UNIQUE",
            Self::PatternNotFound => "PATTERN_NOT_FOUND",
            Self::InvalidPath => "INVALID_PATH",
            Self::NetworkError => "NETWORK_ERROR",
            Self::GitError => "GIT_ERROR",
            Self::MemoryKeyNotFound => "MEMORY_KEY_NOT_FOUND",
        };
        write!(f, "{}", s)
    }
}
