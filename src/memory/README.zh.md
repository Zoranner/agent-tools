# 记忆（`memory`）

中文 | [English](README.md)

[← 返回仓库说明](../../README.zh.md)

实现源码：[mod.rs](mod.rs)

## 存储布局（默认）

与 [OpenClaw Memory 概念](https://github.com/openclaw/openclaw/blob/main/docs/concepts/memory.md) 一致的理念：**工作区内纯 Markdown**、按日流水 + 长期总结。路径约定在本库中默认为：

| 用途 | 默认路径（相对工作区根） |
|------|--------------------------|
| 按日追加 | `.agent/memory/YYYY/MM/dd.md`（UTC 日期） |
| 长期总结 | `.agent/memory/MEMORY.md` |

OpenClaw 上游常用的是根下 `memory/YYYY-MM-DD.md` 与根目录 `MEMORY.md`；这里把二者收拢在 **`.agent/memory/`** 下，并按 **年/月/日** 分子目录。可通过 [`MemoryContext::with_memory_dir_relative`](mod.rs) 改掉整个记忆根目录。

每条记录为一个 `### key` 小节，正文为 `content`，块末带有本库写入的 `<!-- agentool-memory: at=… tags=a|b -->` 元数据。

### 全局唯一的 `key`

`memory_write` 会在**整个记忆目录**下扫描已有块：**任意 `.md` 中已存在相同 `### key` 则返回 `MEMORY_KEY_EXISTS`**，不会追加。此时应先用 `memory_read` 查看当前正文（含 `kind` / `file`），再用 `memory_update` 替换「规范块」（见下）。

### 读取与搜索时的「总结优先」

- **`memory_read`**：若 `MEMORY.md` 中存在该 `key` 的块，则只在**这些块**中取 `at` 最新的一条；否则在**全部日记忆**中取 `at` 最新。返回里的 **`kind`** 为 `"summary"` 或 `"daily"`。
- **`memory_search`**：每条结果含 **`kind`**；**先列出所有 `summary`，再列出 `daily`**；同一层级内按 `at` 降序（无 `at` 的块视为最旧）。

---

## `memory_write`

**新建**一条记忆（`key` 全局不得重复）。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 单行 `###` 标题键 |
| `content` | `string` | 是 | 正文 |
| `target` | `string` | 否 | `daily`（默认）→ 当日 `YYYY/MM/dd.md`；`summary` → `MEMORY.md` |
| `tags` | `string[]` | 否 | 写入元数据 |

**返回**：`key`、`target`、`path`（相对记忆根目录）。

---

## `memory_update`

用新正文**替换**当前「规范块」（解析规则与 `memory_read` 相同）：在**原文件**中删除该块的字节区间，并在**该文件末尾**追加新块（新 `at`、可更新 `tags`）。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 已存在的键 |
| `content` | `string` | 是 | 新正文 |
| `tags` | `string[]` | 否 | 新标签（覆盖元数据） |

**返回**：`key`、`path`、`kind`（`summary` / `daily`，表示被替换块原先所在类型）。

若 `key` 不存在： `MEMORY_KEY_NOT_FOUND`。

---

## `memory_read`

见上文「总结优先」。返回字段：`key`、`content`、`tags`、`file`、`kind`、`created_at`、`updated_at`（当前块元数据中的 `at`；追加型更新后二者一致）。

---

## `memory_search`

不区分大小写的子串匹配（`key` 与 `content`）。结果含 **`kind`**，**summary 排在 daily 之前**。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 关键词；可为空，仅按 `tags` 过滤 |
| `tags` | `string[]` | 否 | 块元数据须包含**全部**所列标签 |
| `limit` | `number` | 否 | 默认 10，最大 100 |

## 错误码

| 错误码 | 说明 |
|--------|------|
| `INVALID_PATH` | 必填字符串参数缺失或类型错误（见库内 `core::json`） |
| `MEMORY_KEY_EXISTS` | `memory_write` 时 `key` 已在任意记忆文件中出现 |
| `MEMORY_KEY_NOT_FOUND` | `memory_read` / `memory_update` 无匹配块 |
| `MEMORY_INVALID_KEY` | `key` 为空、仅空白或含换行 |
| `MEMORY_INVALID_TARGET` | `target` 不是 `daily` / `summary` |
| `MEMORY_STORAGE_ERROR` | 目录/文件读写失败或内部跨度错误 |

## 与手写 Markdown 的兼容

解析器以 `### ` 行作为块起点。手写块若无 `<!-- agentool-memory: ... -->`，`at` 为空，在排序中视为最旧；`memory_read` 在「多候选」时可能不如带时间戳的块优先。
