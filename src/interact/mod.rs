//! Human-in-the-loop tools: [`ask`](AskTool), [`confirm`](ConfirmTool), [`notify`](NotifyTool).
//!
//! Actual I/O is provided by [`InteractBackend`]; use [`InteractContext::unsupported`] only when
//! you will not call `ask`/`confirm`, or inject [`StubInteractBackend`](backends::StubInteractBackend) for tests.

mod backend;
mod backends;
mod error;
mod ops;
mod tools;

use std::sync::Arc;

pub use backend::{InteractBackend, NotifyLevel, UnsupportedInteractBackend};
pub use backends::StubInteractBackend;
pub use tools::{all_tools, AskTool, ConfirmTool, NotifyTool};

/// Holds the [`InteractBackend`] used by all interact tools.
#[derive(Clone)]
pub struct InteractContext {
    pub backend: Arc<dyn InteractBackend>,
}

impl InteractContext {
    pub fn new(backend: Arc<dyn InteractBackend>) -> Self {
        Self { backend }
    }

    /// [`UnsupportedInteractBackend`]: `ask` / `confirm` error; `notify` returns `sent: false`.
    pub fn unsupported() -> Self {
        Self {
            backend: Arc::new(UnsupportedInteractBackend),
        }
    }
}

impl std::fmt::Debug for InteractContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InteractContext")
            .field("backend", &"<dyn InteractBackend>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn unsupported_ask_and_confirm_err_notify_false() {
        let ctx = Arc::new(InteractContext::unsupported());
        let tools = all_tools(ctx);

        let ask = tools.iter().find(|t| t.name() == "ask").unwrap();
        let e = ask.execute(json!({ "question": "hi?" })).await.unwrap_err();
        assert_eq!(e.code, "INTERACT_NOT_SUPPORTED");

        let confirm = tools.iter().find(|t| t.name() == "confirm").unwrap();
        let e2 = confirm
            .execute(json!({ "message": "sure?" }))
            .await
            .unwrap_err();
        assert_eq!(e2.code, "INTERACT_NOT_SUPPORTED");

        let notify = tools.iter().find(|t| t.name() == "notify").unwrap();
        let n = notify
            .execute(json!({ "message": "ping", "level": "warning" }))
            .await
            .unwrap();
        assert_eq!(n["data"]["sent"], false);
    }

    #[tokio::test]
    async fn stub_backend_roundtrip() {
        let ctx = Arc::new(InteractContext::new(Arc::new(StubInteractBackend {
            answer: "blue".into(),
            confirmed: false,
            notify_sent: true,
        })));
        let tools = all_tools(ctx);

        let ask = tools.iter().find(|t| t.name() == "ask").unwrap();
        let a = ask.execute(json!({ "question": "color?" })).await.unwrap();
        assert_eq!(a["data"]["answer"], "blue");

        let confirm = tools.iter().find(|t| t.name() == "confirm").unwrap();
        let c = confirm.execute(json!({ "message": "go?" })).await.unwrap();
        assert_eq!(c["data"]["confirmed"], false);

        let notify = tools.iter().find(|t| t.name() == "notify").unwrap();
        let n = notify.execute(json!({ "message": "done" })).await.unwrap();
        assert_eq!(n["data"]["sent"], true);
    }

    #[tokio::test]
    async fn invalid_notify_level() {
        let ctx = Arc::new(InteractContext::unsupported());
        let notify = all_tools(ctx)
            .into_iter()
            .find(|t| t.name() == "notify")
            .unwrap();
        let e = notify
            .execute(json!({ "message": "x", "level": "loud" }))
            .await
            .unwrap_err();
        assert_eq!(e.code, "INTERACT_INVALID_PARAM");
    }
}
