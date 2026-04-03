# Markdown (`md`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs)

## `extract_toc`

Extract an **ATX heading** table of contents (`#` … `######`). Lines inside **fenced code blocks** (`` ``` `` / `~~~`) are ignored.

Path resolution matches `fs`: relative paths join to [`MdContext`](mod.rs) workspace root; with `allow_outside_root == false`, paths must stay inside the root.

| Parameter | Type | Required |
|-----------|------|----------|
| `path` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `path` | `string` |
| `toc` | `TocItem[]` |
| `toc[].level` | `number` | 1–6 |
| `toc[].title` | `string` |
| `toc[].line` | `number` | 1-based line |

---

## `markdown_stats`

Counts **non-whitespace characters**, paragraphs, ATX headings, and total lines. `characters` / `paragraphs` **exclude fenced code blocks**; `headings` counts ATX headings outside fences; `lines` includes everything.

Paragraphs are split on blank lines (`\n\n`). There is **no word count**—use `characters` for CJK/Latin without a single tokenizer definition.

| Parameter | Type | Required |
|-----------|------|----------|
| `path` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `path` | `string` |
| `characters` | `number` |
| `paragraphs` | `number` |
| `headings` | `number` |
| `lines` | `number` |

## Error codes

Similar to `fs` (see [`error.rs`](error.rs)):

| Code | Meaning |
|------|---------|
| `FILE_NOT_FOUND` | Missing path or cannot resolve in sandbox |
| `PERMISSION_DENIED` | Cannot read |
| `INVALID_PATH` | Empty path, outside root, not a file, non-UTF-8, or other I/O failure |
