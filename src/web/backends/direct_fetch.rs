use async_trait::async_trait;
use reqwest::{Client, Url};

use crate::{ToolError, ToolErrorCode};

use super::super::backend::WebFetchBackend;
use super::super::html::extract_title;
use super::super::types::WebFetchResult;

fn tool_error(code: ToolErrorCode, message: impl Into<String>) -> ToolError {
    ToolError {
        code,
        message: message.into(),
    }
}

/// Fetch URL with [`reqwest`], convert HTML to Markdown using [`htmd`].
#[derive(Debug, Default, Clone, Copy)]
pub struct DirectFetchBackend;

#[async_trait]
impl WebFetchBackend for DirectFetchBackend {
    async fn fetch(&self, client: &Client, url: &Url) -> Result<WebFetchResult, ToolError> {
        let resp = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| tool_error(ToolErrorCode::NetworkError, e.to_string()))?;

        let final_url = resp.url().to_string();
        let status = resp.status();
        if !status.is_success() {
            return Err(tool_error(
                ToolErrorCode::NetworkError,
                format!("HTTP {}", status.as_u16()),
            ));
        }

        let html = resp
            .text()
            .await
            .map_err(|e| tool_error(ToolErrorCode::NetworkError, e.to_string()))?;

        let title = extract_title(&html).unwrap_or_default();
        let content = htmd::convert(&html).map_err(|e| {
            tool_error(
                ToolErrorCode::NetworkError,
                format!("HTML to Markdown conversion failed: {e}"),
            )
        })?;

        Ok(WebFetchResult {
            content,
            title,
            url: final_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::web::html::extract_title;

    #[test]
    fn extract_title_basic() {
        let html = "<html><title>Hi &amp; there</title></html>";
        assert_eq!(extract_title(html).as_deref(), Some("Hi & there"));
    }
}
