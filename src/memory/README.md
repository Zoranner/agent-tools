# 记忆（`memory`）

[← 返回仓库说明](../../README.md)

实现源码：[mod.rs](mod.rs)

## `memory_write`

存储一条记忆条目。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 记忆标识符 |
| `content` | `string` | 是 | 记忆内容 |
| `tags` | `string[]` | 否 | 标签，用于分类和搜索 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 记忆标识符 |

---

## `memory_read`

读取指定记忆条目。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 记忆标识符 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 记忆标识符 |
| `content` | `string` | 记忆内容 |
| `tags` | `string[]` | 标签列表 |
| `created_at` | `string` | 创建时间（ISO 8601） |
| `updated_at` | `string` | 更新时间（ISO 8601） |

---

## `memory_search`

按关键词搜索历史记忆。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 搜索关键词 |
| `tags` | `string[]` | 否 | 按标签过滤 |
| `limit` | `number` | 否 | 返回数量，默认 10 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `MemoryItem[]` | 匹配的记忆列表 |
| `results[].key` | `string` | 记忆标识符 |
| `results[].content` | `string` | 记忆内容 |
| `results[].tags` | `string[]` | 标签列表 |

## 错误码

本模块工具可能返回的 `error.code` 如下（与 [`../error.rs`](../error.rs) 中 `ToolErrorCode` 一致）。

| 错误码 | 说明 |
|--------|------|
| `MEMORY_KEY_NOT_FOUND` | `memory_read` 等按 key 访问时条目不存在 |
