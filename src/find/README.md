# Find (`find`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

> The `find` feature is **workspace search**, distinct from web search (`web_search`). Tool names follow Unix habits: `grep_search`, `glob_search`.
>
> **`FindContext`**
>
> Like `fs`, find tools share a context: `FindContext::new(root)` canonicalizes `root` as the default scan root; `None` uses the process current directory at construction.
>
> `path` is optional for `grep_search` / `glob_search` (defaults to that root). Relative `path` joins to the default root; absolute paths resolve on the filesystem. `path` must exist as a file or directory.
>
> Traversal skips directories named `.git`. `grep_search` skips files that are not valid UTF-8.

## `grep_search`

Search file contents with a **regular expression** (Rust `regex` syntax). Plain keywords work as literals; escape metacharacters like `.` `(` when needed.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `pattern` | `string` | yes | Regex; invalid syntax → `INVALID_PATTERN` |
| `path` | `string` | no | Root dir or single file; default is `FindContext` root |
| `glob` | `string` | no | Path glob under root, e.g. `**/*.md`; invalid → `INVALID_PATTERN` |
| `ignore_case` | `boolean` | no | Default `false` |

**Returns**

| Field | Type |
|-------|------|
| `matches` | `Match[]` |
| `matches[].file` | `string` |
| `matches[].line` | `number` |
| `matches[].content` | `string` |

---

## `glob_search`

Match files by glob pattern.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `pattern` | `string` | yes | e.g. `**/*.md`; invalid → `INVALID_PATTERN` |
| `path` | `string` | no | Scan root; default `FindContext` root |

**Returns**

| Field | Type |
|-------|------|
| `files` | `string[]` |

## Error codes

See [`error.rs`](error.rs).

| Code | Meaning |
|------|---------|
| `FILE_NOT_FOUND` | Root or `path` target missing |
| `PERMISSION_DENIED` | Cannot access path |
| `INVALID_PATTERN` | Invalid regex or glob |
| `INVALID_PATH` | Bad arguments, wrong type, walk failure |
