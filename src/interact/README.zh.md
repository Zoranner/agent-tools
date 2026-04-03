# 交互（`interact`）

中文 | [English](README.md)

[← 返回仓库说明](../../README.zh.md)

实现源码：[mod.rs](mod.rs)

实际提问、确认、通知的 I/O 由宿主注入 [`InteractBackend`](mod.rs)（终端、桌面、LSP、MCP 等）。[`InteractContext::new`](mod.rs) 传入 `Arc<dyn InteractBackend>`；若暂时不接宿主，可用 [`InteractContext::unsupported`](mod.rs)（`interact_ask` / `interact_confirm` 会报错，`interact_notify` 返回 `sent: false`）。单元测试可使用 [`StubInteractBackend`](backends/mod.rs)。

## `interact_ask`

向用户提问，等待并返回回答。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `question` | `string` | 是 | 问题内容 |
| `options` | `string[]` | 否 | 非空时为单选；缺省或空数组为自由回答 |
| `timeout` | `number` | 否 | 超时秒数，由后端解释并实现 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `answer` | `string` | 回答内容 |

---

## `interact_confirm`

向用户请求确认，等待是/否结果。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `message` | `string` | 是 | 确认提示内容 |
| `default` | `boolean` | 否 | 超时或无响应时的默认值，默认 `false` |
| `timeout` | `number` | 否 | 超时秒数 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `confirmed` | `boolean` | 是否确认 |

---

## `interact_notify`

向用户发送通知，无需等待回复。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `message` | `string` | 是 | 通知内容 |
| `level` | `"info" \| "warning" \| "error"` | 否 | 通知级别，默认 `"info"` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `sent` | `boolean` | 后端是否报告发送成功（无后端时多为 `false`） |

## 错误码

| 错误码 | 说明 |
|--------|------|
| `INVALID_PATH` | 必填字符串参数缺失或类型错误（见库内 `core::json`） |
| `INTERACT_NOT_SUPPORTED` | 当前上下文未配置可用的 `InteractBackend`（默认 `unsupported` 下 `interact_ask` / `interact_confirm`） |
| `INTERACT_TIMEOUT` | 自定义后端在超时时可直接构造 `ToolError`，`code` 为该字符串 |
| `INTERACT_CANCELLED` | 用户取消等，同上，`code` 为该字符串 |
| `INTERACT_INVALID_PARAM` | 如 `level` 非法枚举值 |
