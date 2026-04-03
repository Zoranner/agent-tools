# 查找（`find`）

中文 | [English](README.md)

[← 返回仓库说明](../../README.zh.md)

> 模块名 `find` 表示**工作区内查找**，与网络检索（`web_search`）区分；工具名仍沿用类 Unix 习惯：`grep_search`、`glob_search`。
>
> **`FindContext`**
>
> 与 `fs` 类似，查找工具共享一个上下文：`FindContext::new(root)` 将 `root` canonicalize 为默认扫描根；`root` 为 `None` 时使用构造瞬间的进程当前目录。
>
> `grep_search` / `glob_search` 的 `path` 可省略，此时从该默认根扫描。传入相对 `path` 时相对于默认根拼接；传入绝对路径时按文件系统解析。`path` 须指向已存在的目录或文件。
>
> 遍历时不会进入名为 `.git` 的目录。`grep_search` 跳过无法按 UTF-8 解码的文件。

## `grep_search`

按 **正则表达式**（Rust `regex` crate 语法）搜索文件内容；简单关键字可直接写普通字符，含 `.` `(` 等时需按正则规则转义。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | 正则表达式；语法非法时返回 `INVALID_PATTERN` |
| `path` | `string` | 否 | 扫描根目录或单个文件，默认 `FindContext` 的根 |
| `glob` | `string` | 否 | 相对根的路径 glob，如 `**/*.md`；非法时返回 `INVALID_PATTERN` |
| `ignore_case` | `boolean` | 否 | 是否忽略大小写，默认 `false` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `matches` | `Match[]` | 匹配列表 |
| `matches[].file` | `string` | 文件路径 |
| `matches[].line` | `number` | 行号 |
| `matches[].content` | `string` | 匹配行内容 |

---

## `glob_search`

按文件名 Glob 模式匹配文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | Glob 模式，如 `**/*.md`；非法时返回 `INVALID_PATTERN` |
| `path` | `string` | 否 | 扫描根目录，默认 `FindContext` 的根 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `files` | `string[]` | 匹配的文件路径列表 |

## 错误码

本模块工具可能返回的 `error.code` 如下（定义见 [`error.rs`](error.rs)）。

| 错误码 | 说明 |
|--------|------|
| `FILE_NOT_FOUND` | 扫描根路径或 `path` 指向的目标不存在 |
| `PERMISSION_DENIED` | 解析或访问路径时权限不足 |
| `INVALID_PATTERN` | `grep_search` 的正则或 `glob` / `glob_search` 的 glob 语法无效 |
| `INVALID_PATH` | 参数缺失/非法、目标不是预期类型、目录遍历失败等 |
