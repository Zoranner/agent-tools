//! Network tools: [`web_search`](WebSearchTool) and [`web_fetch`](WebFetchTool).
//!
//! Defaults: [`DuckDuckGoSearchBackend`](backends::DuckDuckGoSearchBackend) for search and
//! [`DirectFetchBackend`](backends::DirectFetchBackend) (HTTP + [`htmd`]) for fetch.
//! Replace them via [`WebContextBuilder`] to plug in Brave, Tavily, Jina Reader, etc.

mod backend;
mod backends;
mod error;
mod html;
mod ops;
mod tools;
mod types;

use std::sync::Arc;

pub use backend::{WebFetchBackend, WebSearchBackend};
pub use backends::{DirectFetchBackend, DuckDuckGoSearchBackend};
#[cfg(feature = "tavily")]
pub use backends::{TavilyFetchBackend, TavilySearchBackend};
pub use tools::{all_tools, WebFetchTool, WebSearchTool};
pub use types::{WebFetchResult, WebSearchResult};

use reqwest::Client;

/// Shared HTTP client and pluggable search/fetch backends.
#[derive(Clone)]
pub struct WebContext {
    pub client: Client,
    search_backend: Arc<dyn WebSearchBackend>,
    fetch_backend: Arc<dyn WebFetchBackend>,
}

impl WebContext {
    /// Default client, DuckDuckGo search, direct HTML→Markdown fetch.
    pub fn new() -> Result<Self, reqwest::Error> {
        WebContextBuilder::new().build()
    }

    /// Use a custom [`Client`] with default backends.
    pub fn with_client(client: Client) -> Self {
        WebContext {
            client,
            search_backend: Arc::new(DuckDuckGoSearchBackend),
            fetch_backend: Arc::new(DirectFetchBackend),
        }
    }

    /// Full control without using the builder.
    pub fn from_parts(
        client: Client,
        search: Arc<dyn WebSearchBackend>,
        fetch: Arc<dyn WebFetchBackend>,
    ) -> Self {
        Self {
            client,
            search_backend: search,
            fetch_backend: fetch,
        }
    }

    pub(crate) fn search_backend(&self) -> &dyn WebSearchBackend {
        self.search_backend.as_ref()
    }

    pub(crate) fn fetch_backend(&self) -> &dyn WebFetchBackend {
        self.fetch_backend.as_ref()
    }
}

impl std::fmt::Debug for WebContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebContext")
            .field("client", &"<reqwest::Client>")
            .field("search_backend", &"<dyn WebSearchBackend>")
            .field("fetch_backend", &"<dyn WebFetchBackend>")
            .finish()
    }
}

/// Configure [`WebContext`]: optional client, custom search/fetch backends.
#[derive(Default)]
pub struct WebContextBuilder {
    client: Option<Client>,
    search_backend: Option<Arc<dyn WebSearchBackend>>,
    fetch_backend: Option<Arc<dyn WebFetchBackend>>,
}

impl WebContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the HTTP client (timeouts, proxy, TLS, etc.).
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Use a custom search implementation (`Arc` for shared ownership).
    pub fn search_backend(mut self, backend: Arc<dyn WebSearchBackend>) -> Self {
        self.search_backend = Some(backend);
        self
    }

    /// Convenience: wrap `backend` in [`Arc`].
    pub fn search<B>(mut self, backend: B) -> Self
    where
        B: WebSearchBackend + 'static,
    {
        self.search_backend = Some(Arc::new(backend));
        self
    }

    /// Use a custom fetch implementation.
    pub fn fetch_backend(mut self, backend: Arc<dyn WebFetchBackend>) -> Self {
        self.fetch_backend = Some(backend);
        self
    }

    /// Convenience: wrap `backend` in [`Arc`].
    pub fn fetch<B>(mut self, backend: B) -> Self
    where
        B: WebFetchBackend + 'static,
    {
        self.fetch_backend = Some(Arc::new(backend));
        self
    }

    pub fn build(self) -> Result<WebContext, reqwest::Error> {
        let client = match self.client {
            Some(c) => c,
            None => Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(15))
                .user_agent(concat!(
                    "agentool/",
                    env!("CARGO_PKG_VERSION"),
                    " (+https://github.com/Zoranner/agent-tools)"
                ))
                .redirect(reqwest::redirect::Policy::limited(8))
                .build()?,
        };

        let search_backend = self
            .search_backend
            .unwrap_or_else(|| Arc::new(DuckDuckGoSearchBackend));
        let fetch_backend = self
            .fetch_backend
            .unwrap_or_else(|| Arc::new(DirectFetchBackend));

        Ok(WebContext {
            client,
            search_backend,
            fetch_backend,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    #[derive(Debug)]
    struct StubSearch;

    #[async_trait]
    impl WebSearchBackend for StubSearch {
        async fn search(
            &self,
            _client: &Client,
            query: &str,
            _limit: usize,
        ) -> Result<Vec<WebSearchResult>, crate::tool::ToolError> {
            Ok(vec![WebSearchResult {
                title: "stub".to_string(),
                url: "https://example.test/".to_string(),
                snippet: query.to_string(),
            }])
        }
    }

    #[tokio::test]
    async fn web_fetch_httpbin_html() {
        let ctx = Arc::new(WebContext::new().expect("web client"));
        let tools = all_tools(ctx);
        let fetch = tools.iter().find(|t| t.name() == "web_fetch").unwrap();
        let out = fetch
            .execute(json!({ "url": "https://httpbin.org/html" }))
            .await
            .expect("fetch");
        assert_eq!(out["success"], true);
        let data = &out["data"];
        assert!(data["content"].as_str().unwrap().contains("Herman"));
        assert!(data["url"].as_str().unwrap().contains("httpbin.org"));
    }

    #[tokio::test]
    async fn custom_search_backend() {
        let ctx = Arc::new(
            WebContextBuilder::new()
                .search(StubSearch)
                .build()
                .expect("ctx"),
        );
        let tools = all_tools(ctx);
        let s = tools.iter().find(|t| t.name() == "web_search").unwrap();
        let out = s
            .execute(json!({ "query": "hello" }))
            .await
            .expect("search");
        let r = out["data"]["results"].as_array().unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0]["snippet"], "hello");
    }
}
