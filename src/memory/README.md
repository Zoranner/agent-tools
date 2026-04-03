# 记忆（`memory`）

[← 返回仓库说明](../../README.md)

实现源码：[mod.rs](mod.rs)

## 存储布局（默认）

与 [OpenClaw Memory 概念](https://github.com/openclaw/openclaw/blob/main/docs/concepts/memory.md) 一致的理念：**工作区内纯 Markdown**、按日流水 + 长期总结。路径约定在本库中默认为：

| 用途 | 默认路径（相对工作区根） |
|------|--------------------------|
| 按日追加 | `.agent/memory/YYYY/MM/dd.md` |
| 长期总结 | `.agent/memory/MEMORY.md` |

OpenClaw 上游常用的是根下 `memory/YYYY-MM-DD.md` 与根目录 `MEMORY.md`；这里把二者收拢在 **`.agent/memory/`** 下，并按 **年/月/日** 分子目录，便于与 `.agent` 其它产物共存。可通过 [`MemoryContext::with_memory_dir_relative`](mod.rs) 改掉整个记忆根目录。

每日文件与 `MEMORY.md` 中，每条记录为一个 `### key` 小节，正文为你的 `content`，块末带有本库写入的 HTML 注释元数据（时间、标签），供 `memory_read` / `memory_search` 解析。

---

## `memory_write`

追加一条记忆（**不覆盖**整文件，只在目标 Markdown 末尾追加一个块）。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 单行标题键，对应 `### key` |
| `content` | `string` | 是 | 正文 |
| `target` | `string` | 否 | `daily`（默认）→ 当日 `YYYY/MM/dd.md`；`summary` → `MEMORY.md` |
| `tags` | `string[]` | 否 | 写入元数据，供 `memory_search` 按标签过滤 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 与参数一致 |
| `target` | `string` | `daily` 或 `summary` |
| `path` | `string` | 相对记忆根目录的文件路径（POSIX 风格 `/`） |

---

## `memory_read`

按 `key` 读取**最新一条**匹配块（在所有 `.md` 中按块内时间戳取最新）。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 与写入时的 `###` 标题一致 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 记忆键 |
| `content` | `string` | 正文 |
| `tags` | `string[]` | 标签 |
| `file` | `string` | 所在文件（相对记忆根目录） |
| `created_at` | `string` | 块时间戳（ISO 8601，来自写入元数据） |
| `updated_at` | `string` | 当前实现与 `created_at` 相同（仅追加、不就地更新块） |

---

## `memory_search`

在记忆根目录下递归扫描所有 `.md`，对 **key 与 content** 做不区分大小写的子串匹配。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 关键词；可为空字符串，仅按 `tags` 过滤 |
| `tags` | `string[]` | 否 | 要求块元数据中包含**全部**所列标签 |
| `limit` | `number` | 否 | 返回条数，默认 10，最大 100 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `array` | 匹配块列表 |
| `results[].key` | `string` | 键 |
| `results[].content` | `string` | 正文 |
| `results[].tags` | `string[]` | 标签 |
| `results[].file` | `string` | 来源文件（相对记忆根目录） |

## 错误码

| 错误码 | 说明 |
|--------|------|
| `INVALID_PATH` | 必填字符串参数缺失或类型错误（见库内 `core::json`） |
| `MEMORY_KEY_NOT_FOUND` | `memory_read` 时无匹配块 |
| `MEMORY_INVALID_KEY` | `key` 为空、仅空白，或含换行 |
| `MEMORY_INVALID_TARGET` | `target` 不是 `daily` / `summary` |
| `MEMORY_STORAGE_ERROR` | 创建目录、读写文件失败，或记忆根路径无效 |

## 与手写 Markdown 的兼容

解析器只认以 `### ` 开头的块，且推荐块末带有本库写入的 `<!-- agentool-memory: ... -->` 行以便排序与标签。手写内容若不含该注释，`memory_read` / `memory_search` 仍可能匹配正文，但时间戳可能为空，排序行为会较弱。
