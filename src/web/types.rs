/// One hit from [`super::WebSearchBackend`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Fetched page from [`super::WebFetchBackend`].
#[derive(Debug, Clone)]
pub struct WebFetchResult {
    pub content: String,
    pub title: String,
    pub url: String,
}
