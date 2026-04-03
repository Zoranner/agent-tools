# Web (`web`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs), [backend.rs](backend.rs), [types.rs](types.rs), [backends/](backends/), [ops.rs](ops.rs), [tools.rs](tools.rs)

**`WebContext` and pluggable backends**

- `WebContext::new()?`: default `reqwest::Client` (~30s timeout, 15s connect, up to 8 redirects, `agentool` User-Agent) + [`DuckDuckGoSearchBackend`](backends/duckduckgo.rs) + [`DirectFetchBackend`](backends/direct_fetch.rs).
- `WebContext::with_client(client)`: swap the HTTP client only; backends stay default.
- `WebContextBuilder`: combine `.client(...)`, `.search(...)` / `.search_backend(Arc<dyn WebSearchBackend>)`, `.fetch(...)` / `.fetch_backend(Arc<dyn WebFetchBackend>)`; or `WebContext::from_parts(client, search, fetch)`.
- Implement [`WebSearchBackend`](backend.rs) / [`WebFetchBackend`](backend.rs) returning [`WebSearchResult`](types.rs) / [`WebFetchResult`](types.rs); errors are [`ToolError`](../tool.rs) (often `NETWORK_ERROR`).

Custom search example:

```rust
use std::sync::Arc;
use async_trait::async_trait;
use reqwest::Client;
use agentool::web::{WebContextBuilder, WebSearchBackend, WebSearchResult, all_tools};
use agentool::ToolError;

struct MySearch;

#[async_trait]
impl WebSearchBackend for MySearch {
    async fn search(
        &self,
        _client: &Client,
        query: &str,
        limit: usize,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        // Brave / Tavily / self-hosted SearXNG, etc.
        let _ = (query, limit);
        Ok(vec![])
    }
}

let ctx = Arc::new(WebContextBuilder::new().search(MySearch).build()?);
let _tools = all_tools(ctx);
```

**Default search (DuckDuckGo)**

`DuckDuckGoSearchBackend`: Instant Answer JSON; falls back to HTML parsing (`result__a`, `uddg=` decoding). Page layout may change—use a paid API or your own index in production.

**Default fetch (direct + htmd)**

`DirectFetchBackend`: `web_fetch` allows only `http` / `https` (checked in `ops`); HTML is converted to Markdown with `htmd`. Swap in Jina Reader, headless browser, etc. via a custom [`WebFetchBackend`](backend.rs).

## `web_search`

Search the web; returns snippets.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `query` | `string` | yes | Query string |
| `limit` | `number` | no | Default 5 |

**Returns**

| Field | Type |
|-------|------|
| `results` | `Result[]` |
| `results[].title` | `string` |
| `results[].url` | `string` |
| `results[].snippet` | `string` |

---

## `web_fetch`

Fetch a URL and convert HTML to Markdown.

| Parameter | Type | Required |
|-----------|------|----------|
| `url` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `content` | `string` |
| `title` | `string` |
| `url` | `string` | Final URL after redirects |

## Error codes

- Missing/invalid string args: [`core/json.rs`](../core/json.rs) → `INVALID_PATH`
- Network / `limit`: [`error.rs`](error.rs) `WebErrorCode` → `NETWORK_ERROR`

| Code | Meaning |
|------|---------|
| `INVALID_PATH` | Bad `query` / `url` type or missing required string |
| `NETWORK_ERROR` | Bad `limit`, fetch/search failure, non-success HTTP, non-http(s) URL, etc. |
