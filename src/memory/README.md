# Memory (`memory`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs)

## Default layout

Aligned with [OpenClaw memory concepts](https://github.com/openclaw/openclaw/blob/main/docs/concepts/memory.md): **Markdown in the workspace**, daily log + long-lived summary. Defaults:

| Role | Path (relative to workspace root) |
|------|-----------------------------------|
| Daily append | `.agent/memory/YYYY/MM/dd.md` (UTC date) |
| Summary | `.agent/memory/MEMORY.md` |

Upstream OpenClaw often uses `memory/YYYY-MM-DD.md` and root `MEMORY.md`; here both live under **`.agent/memory/`** with **year/month/day** folders. Override the root with [`MemoryContext::with_memory_dir_relative`](mod.rs).

Each entry is a `### key` section; body is `content`; blocks end with `<!-- agentool-memory: at=… tags=a|b -->`.

### Globally unique `key`

`memory_write` scans the whole memory tree: if any `.md` already has the same `### key`, it returns **`MEMORY_KEY_EXISTS`** and does not append. Use `memory_read` (see `kind` / `file`) then `memory_update` for the canonical block.

### Summary-first read/search

- **`memory_read`**: if `MEMORY.md` has blocks for `key`, pick the newest by `at` among those; else pick newest among **daily** files. Response includes **`kind`**: `"summary"` or `"daily"`.
- **`memory_search`**: each hit has **`kind`**; **all `summary` rows first**, then `daily`; within a tier, sort by `at` descending (missing `at` = oldest).

---

## `memory_write`

**Create** a new memory (`key` must be new globally).

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `key` | `string` | yes | Single-line `###` title |
| `content` | `string` | yes | Body |
| `target` | `string` | no | `daily` (default) → today `YYYY/MM/dd.md`; `summary` → `MEMORY.md` |
| `tags` | `string[]` | no | Metadata |

**Returns**: `key`, `target`, `path` (relative to memory root).

---

## `memory_update`

Replace the **canonical** block (same rules as `memory_read`): remove its byte span **in place**, append the new block at **end of that file** (new `at`, optional `tags`).

| Parameter | Type | Required |
|-----------|------|----------|
| `key` | `string` | yes |
| `content` | `string` | yes |
| `tags` | `string[]` | no | Replaces metadata |

**Returns**: `key`, `path`, `kind` (`summary` / `daily` for the replaced block).

Missing key → `MEMORY_KEY_NOT_FOUND`.

---

## `memory_read`

See “Summary-first” above. Fields: `key`, `content`, `tags`, `file`, `kind`, `created_at`, `updated_at` (from block `at`; equal after append-style updates).

---

## `memory_search`

Case-insensitive substring match on `key` and `content`. Results include **`kind`**; **summary before daily**.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `query` | `string` | yes | May be empty to filter by `tags` only |
| `tags` | `string[]` | no | Block must contain **all** listed tags |
| `limit` | `number` | no | Default 10, max 100 |

## Error codes

| Code | Meaning |
|------|---------|
| `INVALID_PATH` | Missing/invalid string args (`core::json`) |
| `MEMORY_KEY_EXISTS` | `memory_write` duplicate `key` |
| `MEMORY_KEY_NOT_FOUND` | `memory_read` / `memory_update` no block |
| `MEMORY_INVALID_KEY` | Empty/whitespace/multiline `key` |
| `MEMORY_INVALID_TARGET` | `target` not `daily` / `summary` |
| `MEMORY_STORAGE_ERROR` | I/O or internal span error |

## Hand-written Markdown

Parser starts blocks at `### ` lines. Blocks without `<!-- agentool-memory: ... -->` have empty `at` (sorts oldest); `memory_read` may deprioritize them when multiple candidates exist.
