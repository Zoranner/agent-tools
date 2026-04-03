use async_trait::async_trait;

use crate::tool::ToolError;

use super::error::{tool_error, InteractErrorCode};

/// 通知级别（`notify`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotifyLevel {
    Info,
    Warning,
    Error,
}

impl NotifyLevel {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "info" => Ok(Self::Info),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            other => Err(tool_error(
                InteractErrorCode::InvalidParam,
                format!("`level` must be \"info\", \"warning\", or \"error\", got {other:?}"),
            )),
        }
    }
}

/// 宿主实现的交互 I/O（终端、GUI、LSP、MCP 等）。
#[async_trait]
pub trait InteractBackend: Send + Sync {
    /// 提问；`options` 为 `Some` 且非空时表示单选。
    async fn ask(
        &self,
        question: &str,
        options: Option<Vec<String>>,
        timeout_secs: Option<u64>,
    ) -> Result<String, ToolError>;

    async fn confirm(
        &self,
        message: &str,
        default: bool,
        timeout_secs: Option<u64>,
    ) -> Result<bool, ToolError>;

    async fn notify(&self, message: &str, level: NotifyLevel) -> Result<bool, ToolError>;
}

/// 占位后端：`ask` / `confirm` 返回 [`INTERACT_NOT_SUPPORTED`](InteractErrorCode::NotSupported)；`notify` 返回 `Ok(false)`。
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedInteractBackend;

#[async_trait]
impl InteractBackend for UnsupportedInteractBackend {
    async fn ask(
        &self,
        _question: &str,
        _options: Option<Vec<String>>,
        _timeout_secs: Option<u64>,
    ) -> Result<String, ToolError> {
        Err(tool_error(
            InteractErrorCode::NotSupported,
            "no InteractBackend configured; provide one via InteractContext::new",
        ))
    }

    async fn confirm(
        &self,
        _message: &str,
        _default: bool,
        _timeout_secs: Option<u64>,
    ) -> Result<bool, ToolError> {
        Err(tool_error(
            InteractErrorCode::NotSupported,
            "no InteractBackend configured; provide one via InteractContext::new",
        ))
    }

    async fn notify(&self, _message: &str, _level: NotifyLevel) -> Result<bool, ToolError> {
        Ok(false)
    }
}
