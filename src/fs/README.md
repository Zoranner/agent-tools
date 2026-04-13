# Filesystem (`fs`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

> **Paths and sandboxing**
>
> All `fs` tools share one `FsContext`:
>
> | Mode | Behavior |
> |------|----------|
> | `FsContext::new(root, false)` (default) | **Sandbox**: relative paths join to the workspace root; the resolved path must stay under that root (including the root itself) or `INVALID_PATH` is returned |
> | `FsContext::new(root, true)` | **Relaxed**: relative paths still join to the workspace root like sandbox mode; **no** check that the result stays inside the root, so absolute paths or normalized `..` may escape |
>
> If `root` is `None`, the process current directory is canonicalized at construction time. Paths are lexically normalized for `.` / `..` before resolution.

---

## `file_read`

Read a text file with optional line-based paging.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `path` | `string` | yes | Must be a regular file |
| `offset` | `integer` | no | Start line (1-based, ≥ 1) |
| `limit` | `integer` | no | Max lines (non-negative integer) |

`offset` and `limit` must be JSON integers, not floats.

**Returns**

| Field | Type | Notes |
|-------|------|--------|
| `content` | `string` | Raw newlines preserved (CRLF not normalized) |
| `total_lines` | `number` | Logical line count |

---

## `file_write`

Write a file; create if missing, overwrite if present. Missing parent directories are created.

| Parameter | Type | Required |
|-----------|------|----------|
| `path` | `string` | yes |
| `content` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `path` | `string` | Absolute path of the file |

---

## `file_edit`

Replace one occurrence of `old_text`. It must be **unique** in the file and must not be empty.

| Parameter | Type | Required |
|-----------|------|----------|
| `path` | `string` | yes |
| `old_text` | `string` | yes |
| `new_text` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `path` | `string` | Absolute path of the file |

---

## `directory_create`

Create a directory (recursive). No error if it already exists.

| Parameter | Type | Required |
|-----------|------|----------|
| `path` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `path` | `string` | Absolute path of the directory |

---

## `directory_list`

List files and subdirectories, sorted by name ascending.

| Parameter | Type | Required |
|-----------|------|----------|
| `path` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `entries` | `Entry[]` |
| `entries[].name` | `string` |
| `entries[].type` | `"file" \| "directory"` |
| `entries[].size` | `number` | Bytes; 0 for directories |

---

## `file_delete`

Delete a regular file. If the path is a directory, returns `INVALID_PATH`.

| Parameter | Type | Required |
|-----------|------|----------|
| `path` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `path` | `string` | Absolute path of the removed file |

---

## `file_move`

Move or rename a file. Fails if the destination exists; creates parent dirs as needed.

Cross-volume moves fall back to copy + delete; if deleting the source fails, the implementation tries to remove the copied destination and returns an error.

| Parameter | Type | Required |
|-----------|------|----------|
| `source` | `string` | yes |
| `destination` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `source` | `string` |
| `destination` | `string` |

---

## `file_copy`

Copy a file. Fails if the destination exists; creates parent dirs as needed.

| Parameter | Type | Required |
|-----------|------|----------|
| `source` | `string` | yes |
| `destination` | `string` | yes |

**Returns**

| Field | Type |
|-------|------|
| `source` | `string` |
| `destination` | `string` |

## Error codes

Typical `error.code` values (see [`error.rs`](error.rs) `FsErrorCode`; I/O may add system-mapped codes):

| Code | Meaning |
|------|---------|
| `FILE_NOT_FOUND` | Missing file or directory |
| `PERMISSION_DENIED` | Read/write not allowed |
| `FILE_ALREADY_EXISTS` | Destination exists (`file_move` / `file_copy`) |
| `DIRECTORY_NOT_EMPTY` | Non-empty directory or similar I/O semantics |
| `PATTERN_NOT_FOUND` | `old_text` not found in `file_edit` |
| `PATTERN_NOT_UNIQUE` | `old_text` matches more than once |
| `INVALID_PATH` | Empty/invalid path, wrong type (e.g. delete on directory), sandbox violation, or normalization failure |
