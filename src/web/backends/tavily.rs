//! Tavily search and extract backends.
//!
//! # Environment
//!
//! Both backends require the `TAVILY_API_KEY` environment variable to be set
//! at runtime. Obtain a key at <https://app.tavily.com>.

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::tool::ToolError;
use crate::web::error::{tool_error, WebErrorCode};

use super::super::backend::{WebFetchBackend, WebSearchBackend};
use super::super::types::{WebFetchResult, WebSearchResult};

fn api_key() -> Result<String, ToolError> {
    std::env::var("TAVILY_API_KEY").map_err(|_| {
        tool_error(
            WebErrorCode::NetworkError,
            "TAVILY_API_KEY environment variable is not set",
        )
    })
}

// ── Search ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct SearchRequest<'a> {
    api_key: String,
    query: &'a str,
    max_results: usize,
    search_depth: &'a str,
}

#[derive(Deserialize)]
struct SearchResponse {
    results: Vec<SearchHit>,
}

#[derive(Deserialize)]
struct SearchHit {
    title: String,
    url: String,
    #[serde(default)]
    content: String,
}

/// Tavily web search via `POST /search`.
///
/// Requires the `TAVILY_API_KEY` env var at runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct TavilySearchBackend;

#[async_trait]
impl WebSearchBackend for TavilySearchBackend {
    async fn search(
        &self,
        client: &Client,
        query: &str,
        limit: usize,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let body = SearchRequest {
            api_key: api_key()?,
            query,
            max_results: limit,
            search_depth: "basic",
        };

        let resp = client
            .post("https://api.tavily.com/search")
            .json(&body)
            .send()
            .await
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(tool_error(
                WebErrorCode::NetworkError,
                format!("Tavily search HTTP {status}: {text}"),
            ));
        }

        let data: SearchResponse = resp
            .json()
            .await
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        Ok(data
            .results
            .into_iter()
            .take(limit)
            .map(|h| WebSearchResult {
                title: h.title,
                url: h.url,
                snippet: h.content,
            })
            .collect())
    }
}

// ── Extract (fetch) ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ExtractRequest {
    api_key: String,
    urls: Vec<String>,
}

#[derive(Deserialize)]
struct ExtractResponse {
    results: Vec<ExtractHit>,
}

#[derive(Deserialize)]
struct ExtractHit {
    url: String,
    #[serde(default)]
    raw_content: String,
}

/// Tavily content extraction via `POST /extract`.
///
/// Implements [`WebFetchBackend`] — useful as a richer alternative to
/// [`super::DirectFetchBackend`] for pages that are hard to scrape.
///
/// Requires the `TAVILY_API_KEY` env var at runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct TavilyFetchBackend;

#[async_trait]
impl WebFetchBackend for TavilyFetchBackend {
    async fn fetch(&self, client: &Client, url: &Url) -> Result<WebFetchResult, ToolError> {
        let body = ExtractRequest {
            api_key: api_key()?,
            urls: vec![url.to_string()],
        };

        let resp = client
            .post("https://api.tavily.com/extract")
            .json(&body)
            .send()
            .await
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(tool_error(
                WebErrorCode::NetworkError,
                format!("Tavily extract HTTP {status}: {text}"),
            ));
        }

        let data: ExtractResponse = resp
            .json()
            .await
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        let hit = data.results.into_iter().next().ok_or_else(|| {
            tool_error(
                WebErrorCode::NetworkError,
                "Tavily extract returned no results",
            )
        })?;

        Ok(WebFetchResult {
            content: hit.raw_content,
            title: String::new(),
            url: hit.url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_missing() {
        // Ensure api_key() returns an error when the env var is absent.
        std::env::remove_var("TAVILY_API_KEY");
        let err = api_key().unwrap_err();
        assert!(err.message.contains("TAVILY_API_KEY"));
    }
}
