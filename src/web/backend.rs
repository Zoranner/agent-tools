use async_trait::async_trait;
use reqwest::{Client, Url};

use crate::ToolError;

use super::types::{WebFetchResult, WebSearchResult};

/// Pluggable web search (Brave, Tavily, self-hosted SearXNG, etc.).
#[async_trait]
pub trait WebSearchBackend: Send + Sync {
    async fn search(
        &self,
        client: &Client,
        query: &str,
        limit: usize,
    ) -> Result<Vec<WebSearchResult>, ToolError>;
}

/// Pluggable page fetch (direct HTTP, Jina Reader, headless render, etc.).
#[async_trait]
pub trait WebFetchBackend: Send + Sync {
    async fn fetch(&self, client: &Client, url: &Url) -> Result<WebFetchResult, ToolError>;
}
