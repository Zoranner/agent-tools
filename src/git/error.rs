use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum GitErrorCode {
    GitError,
}

impl GitErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::GitError => "GIT_ERROR",
        }
    }
}

pub(crate) fn tool_error(code: GitErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}

pub(crate) fn map_git_err(err: git2::Error) -> ToolError {
    tool_error(GitErrorCode::GitError, err.message().to_string())
}
