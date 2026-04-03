use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum MdErrorCode {
    InvalidPath,
}

impl MdErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPath => "INVALID_PATH",
        }
    }
}

pub(crate) fn tool_error(code: MdErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
