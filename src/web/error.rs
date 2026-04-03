use crate::tool::ToolError;

#[derive(Clone, Copy)]
pub(crate) enum WebErrorCode {
    NetworkError,
}

impl WebErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NetworkError => "NETWORK_ERROR",
        }
    }
}

pub(crate) fn tool_error(code: WebErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code: code.as_str().to_string(),
        message: message.into(),
    }
}
