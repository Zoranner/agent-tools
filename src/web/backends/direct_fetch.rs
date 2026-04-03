use async_trait::async_trait;
use reqwest::{Client, Url};

use crate::tool::ToolError;
use crate::web::error::{tool_error, WebErrorCode};

use super::super::backend::WebFetchBackend;
use super::super::html::extract_title;
use super::super::types::WebFetchResult;

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
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        let final_url = resp.url().to_string();
        let status = resp.status();
        if !status.is_success() {
            return Err(tool_error(
                WebErrorCode::NetworkError,
                format!("HTTP {}", status.as_u16()),
            ));
        }

        let html = resp
            .text()
            .await
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        let title = extract_title(&html).unwrap_or_default();
        let content = htmd::convert(&html).map_err(|e| {
            tool_error(
                WebErrorCode::NetworkError,
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
