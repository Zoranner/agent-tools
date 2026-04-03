# 文档（`md`）

[← 返回仓库说明](../../README.md)

实现源码：[mod.rs](mod.rs)

## `extract_toc`

提取 Markdown 文档的目录结构。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `toc` | `TocItem[]` | 目录列表 |
| `toc[].level` | `number` | 标题层级（1–6） |
| `toc[].title` | `string` | 标题文本 |
| `toc[].line` | `number` | 所在行号 |

---

## `count_words`

统计文档字数及结构信息。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `characters` | `number` | 字符数（不含空格） |
| `words` | `number` | 词数 |
| `paragraphs` | `number` | 段落数 |
| `headings` | `number` | 标题数 |
| `lines` | `number` | 总行数 |

## 错误码

工具按路径读文件时，预期与路径类操作一致；实现落地后可能返回（与 [`../error.rs`](../error.rs) 中 `ToolErrorCode` 一致）：

| 错误码 | 说明 |
|--------|------|
| `FILE_NOT_FOUND` | 给定路径不存在或不是文件 |
| `PERMISSION_DENIED` | 无读取权限 |
| `INVALID_PATH` | 路径非法或 I/O 失败等 |
