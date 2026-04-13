# 文档（`md`）

中文 | [English](README.md)

[← 返回仓库说明](../../README.zh.md)

实现源码：[mod.rs](mod.rs)

## `toc_extract`

提取 Markdown 文档的 **ATX 标题**（`#` … `######`）目录；**围栏代码块**（` ``` ` / `~~~`）内的行不计入目录。

路径解析与 `fs` 一致：相对路径相对 [`MdContext`](mod.rs) 的工作区根目录；`allow_outside_root == false` 时不得越出根目录。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 解析后的文件路径 |
| `toc` | `TocItem[]` | 目录列表 |
| `toc[].level` | `number` | 标题层级（1–6） |
| `toc[].title` | `string` | 标题文本 |
| `toc[].line` | `number` | 所在行号（从 1 起） |

---

## `markdown_inspect`

统计 **非空白字符数**（字数）、段落数、ATX 标题数、总行数。`characters` / `paragraphs` **不包含围栏代码块**内文本；`headings` 仅统计 ATX 标题且同样忽略代码块；`lines` 为全文行数（含代码块与空行）。

段落按 **空行分隔**（`\n\n`）粗略计数。不提供「词数」：中英文分词无统一口径，字数用 `characters` 即可。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 解析后的文件路径 |
| `characters` | `number` | 非空白字符数 |
| `paragraphs` | `number` | 段落数 |
| `headings` | `number` | ATX 标题数 |
| `lines` | `number` | 总行数 |

## 错误码

读路径时的错误与 `fs` 类似（见 [`error.rs`](error.rs)）：

| 错误码 | 说明 |
|--------|------|
| `FILE_NOT_FOUND` | 路径不存在或无法在沙箱内解析 |
| `PERMISSION_DENIED` | 无读取权限 |
| `INVALID_PATH` | 路径为空、越出工作区、`path` 不是普通文件、非 UTF-8 文本、或其它 I/O 失败 |
