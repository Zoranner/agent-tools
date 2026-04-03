# Todo (`todo`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs)

Todos persist in a workspace **JSON** file (default **`.agent/todos.json`**), with the same root/sandbox rules as [`TodoContext`](mod.rs) (`core::path`). Each item has a stable **`id`** (UUID v4) for updates and deletes.

## `todo_add`

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `title` | `string` | yes | Non-empty after trim |
| `description` | `string` | no | |
| `priority` | `string` | no | `low` / `medium` / `high` |
| `tags` | `string[]` | no | `todo_list` can filter by one tag |

**Returns**: `id` (new UUID string). New items have **`status` = `pending`**.

---

## `todo_list`

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `status` | `string` | no | `pending` / `done` / `cancelled` |
| `tag` | `string` | no | Item must include this tag |
| `limit` | `number` | no | Default 50, max 200 |

**Order**: **`pending`**, then **`done`**, then **`cancelled`**; within a status **`high` > `medium` > `low` > none**; then **`updated_at` descending**.

**Returns**: `items` with full objects (`id`, `title`, `description`, `status`, `priority`, `tags`, `created_at`, `updated_at`).

---

## `todo_update`

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `id` | `string` | yes | Target |
| `title` | `string` | no | If set, non-empty |
| `description` | `string` | no | |
| `status` | `string` | no | `pending` / `done` / `cancelled` |
| `priority` | `string` / `null` | no | JSON **`null`** clears priority |
| `tags` | `string[]` | no | If set, **replaces** the whole tag list |

Omitted fields unchanged. `updated_at` refreshed.

---

## `todo_remove`

| Parameter | Type | Required |
|-----------|------|----------|
| `id` | `string` | yes |

**Returns**: `id`, `removed: true`.

## Error codes

| Code | Meaning |
|------|---------|
| `INVALID_PATH` | Missing/invalid string args (`core::json`) |
| `TODO_NOT_FOUND` | Unknown `id` (`todo_update` / `todo_remove`) |
| `TODO_INVALID_INPUT` | Empty title, etc. |
| `TODO_INVALID_STATUS` | Bad `status` |
| `TODO_INVALID_PRIORITY` | Bad `priority` |
| `TODO_STORAGE_ERROR` | JSON I/O, corrupt file, mkdir failure, etc. |
