# 网络（`web`）

[← 返回仓库说明](../../README.md)

实现源码：[mod.rs](mod.rs)、[backend.rs](backend.rs)、[types.rs](types.rs)、[backends/](backends/)、[ops.rs](ops.rs)、[tools.rs](tools.rs)

**`WebContext` 与可插拔后端**

- `WebContext::new()?`：默认 `reqwest::Client`（约 30s 超时、15s 连接超时、最多 8 次重定向、`agentool` User-Agent）+ [`DuckDuckGoSearchBackend`](backends/duckduckgo.rs) + [`DirectFetchBackend`](backends/direct_fetch.rs)。
- `WebContext::with_client(client)`：只换 HTTP 客户端，后端仍为默认。
- `WebContextBuilder`：`.client(...)`、`.search(...)` / `.search_backend(Arc<dyn WebSearchBackend>)`、`.fetch(...)` / `.fetch_backend(Arc<dyn WebFetchBackend>)` 任意组合；`WebContext::from_parts(client, search, fetch)` 亦可一次传入。
- 实现 [`WebSearchBackend`](backend.rs) / [`WebFetchBackend`](backend.rs)，返回 [`WebSearchResult`](types.rs) / [`WebFetchResult`](types.rs)，错误统一为 [`ToolError`](../error.rs)（多为 `NETWORK_ERROR`）。

自定义搜索示例（实现 trait 后用 `WebContextBuilder::new().search(MySearch).build()?`）：

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
        // 调用 Brave / Tavily / 自建 SearXNG 等
        let _ = (query, limit);
        Ok(vec![])
    }
}

let ctx = Arc::new(WebContextBuilder::new().search(MySearch).build()?);
let _tools = all_tools(ctx);
```

**默认搜索（DuckDuckGo）**

`DuckDuckGoSearchBackend`：Instant Answer JSON；不足时 POST HTML 结果页并解析 `result__a`（含 `uddg=` 解码）。页面结构可能变更，生产环境建议换付费 API 或自建索引。

**默认抓取（直连 + htmd）**

`DirectFetchBackend`：`web_fetch` 侧仅允许 `http` / `https`（在 `ops` 中校验）；下载 HTML 后经 `htmd` 转 Markdown。可换为调用 Jina Reader、无头浏览器等自定义 [`WebFetchBackend`](backend.rs)。

## `web_search`

搜索网络，返回相关资料摘要。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 搜索关键词 |
| `limit` | `number` | 否 | 返回结果数量，默认 5 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `Result[]` | 搜索结果列表 |
| `results[].title` | `string` | 页面标题 |
| `results[].url` | `string` | 页面 URL |
| `results[].snippet` | `string` | 内容摘要 |

---

## `web_fetch`

抓取指定网页并转换为 Markdown。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `url` | `string` | 是 | 网页 URL |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `content` | `string` | 转换后的 Markdown 内容 |
| `title` | `string` | 页面标题 |
| `url` | `string` | 实际访问的 URL（含重定向） |

## 错误码

- 字符串必填项缺失/类型错误：[`core/json.rs`](../core/json.rs) → `INVALID_PATH`  
- 网络与 `limit` 校验：[`error.rs`](error.rs) 中 `WebErrorCode` → `NETWORK_ERROR`

| 错误码 | 说明 |
|--------|------|
| `INVALID_PATH` | 缺少或类型不对的 `query` / `url` 等字符串参数 |
| `NETWORK_ERROR` | `limit` 非法、搜索/抓取失败、HTTP 非成功、仅允许 http(s) 等 |
