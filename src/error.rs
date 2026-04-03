//! 工具错误类型与稳定错误码（`Display` 即 JSON 中的 `error.code`）。
//!
//! 按工具集分类的**典型返回场景**说明见各 feature 目录下的 `README.md`（如 `src/fs/README.md`、`src/web/README.md`）文末的「错误码」一节。

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
    /// Invalid user-supplied glob or regular expression.
    InvalidPattern,
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
            Self::InvalidPattern => "INVALID_PATTERN",
            Self::InvalidPath => "INVALID_PATH",
            Self::NetworkError => "NETWORK_ERROR",
            Self::GitError => "GIT_ERROR",
            Self::MemoryKeyNotFound => "MEMORY_KEY_NOT_FOUND",
        };
        write!(f, "{}", s)
    }
}
