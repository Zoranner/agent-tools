use async_trait::async_trait;

use crate::tool::ToolError;

use crate::interact::backend::{InteractBackend, NotifyLevel};

/// 测试或本地开发用：固定返回值，不读 stdin。
#[derive(Debug, Clone)]
pub struct StubInteractBackend {
    pub answer: String,
    pub confirmed: bool,
    pub notify_sent: bool,
}

impl Default for StubInteractBackend {
    fn default() -> Self {
        Self {
            answer: String::from("ok"),
            confirmed: true,
            notify_sent: true,
        }
    }
}

#[async_trait]
impl InteractBackend for StubInteractBackend {
    async fn ask(
        &self,
        _question: &str,
        _options: Option<Vec<String>>,
        _timeout_secs: Option<u64>,
    ) -> Result<String, ToolError> {
        Ok(self.answer.clone())
    }

    async fn confirm(
        &self,
        _message: &str,
        _default: bool,
        _timeout_secs: Option<u64>,
    ) -> Result<bool, ToolError> {
        Ok(self.confirmed)
    }

    async fn notify(&self, _message: &str, _level: NotifyLevel) -> Result<bool, ToolError> {
        Ok(self.notify_sent)
    }
}
