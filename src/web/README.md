# 网络（`web`）

[← 返回仓库说明](../../README.md)

实现源码：[mod.rs](mod.rs)

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

本模块工具可能返回的 `error.code` 如下（与 [`../error.rs`](../error.rs) 中 `ToolErrorCode` 一致）。

| 错误码 | 说明 |
|--------|------|
| `NETWORK_ERROR` | 搜索或抓取请求失败、超时、HTTP 非预期等 |
