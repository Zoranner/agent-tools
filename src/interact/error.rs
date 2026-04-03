use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum InteractErrorCode {
    NotSupported,
    InvalidParam,
}

impl InteractErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NotSupported => "INTERACT_NOT_SUPPORTED",
            Self::InvalidParam => "INTERACT_INVALID_PARAM",
        }
    }
}

pub(crate) fn tool_error(code: InteractErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
