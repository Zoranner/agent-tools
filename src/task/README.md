# Task (`task`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs)

Tasks persist in a **SQLite** database (default **`.agent/tasks.db`**) under the workspace root. Each entity has a stable **`id`** (UUID v4). On startup, any tasks left in `running` state are automatically moved to `blocked` and stale path locks are released.

## Concepts

| Entity | Description |
|--------|-------------|
| **Task** | The top-level unit of work. Has a `kind`, `status`, `owner`, optional `priority`, `tags`, `goal`, `acceptance`, and `risk_level`. |
| **TaskStep** | An ordered sub-step within a task. Steps are appended and tracked individually. |
| **TaskRun** | A single execution attempt of a task (or one of its steps). Records start/end time, status, error, and summary. |
| **PathLock** | An exclusive write lock on a file path. Prevents concurrent writes across tasks. |
| **Checkpoint** | A human-in-the-loop gate. Opening one sets the task to `waiting_checkpoint`; closing it resumes the task. |
| **Artifact** | A reference to a task output: a file path, inline content, or external reference. |

## `task_create`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `title` | `string` | yes | Non-empty after trim |
| `description` | `string` | no | |
| `kind` | `string` | no | `task` / `milestone` / `checkpoint` (default: `task`) |
| `owner` | `string` | no | `agent` / `human` (default: `agent`) |
| `priority` | `string` | no | `low` / `medium` / `high` |
| `tags` | `string[]` | no | |
| `goal` | `string` | no | Task goal description |
| `acceptance` | `string` | no | Acceptance criteria |
| `risk_level` | `string` | no | `low` / `medium` / `high` / `critical` |

**Returns**: `id`. New tasks have **`status` = `backlog`**.

---

## `task_list`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `status` | `string` | no | Filter by status |
| `kind` | `string` | no | Filter by kind |
| `owner` | `string` | no | Filter by owner |
| `tag` | `string` | no | Item must include this tag |
| `limit` | `number` | no | Default 50, max 200 |

**Order**: `updated_at` descending.

**Returns**: `items` array with full task objects.

---

## `task_get`

| Parameter | Type | Required |
|-----------|------|----------|
| `id` | `string` | yes |

**Returns**: Full task object.

---

## `task_update`

Patch semantics — only provided fields change.

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `id` | `string` | yes | |
| `title` | `string` | no | Non-empty if set |
| `description` | `string` | no | |
| `status` | `string` | no | See status values below |
| `priority` | `string` / `null` | no | JSON `null` clears priority |
| `blocked_reason` | `string` / `null` | no | JSON `null` clears |
| `last_error` | `string` / `null` | no | JSON `null` clears |
| `tags` | `string[]` | no | Replaces the whole tag list |
| `risk_level` | `string` / `null` | no | JSON `null` clears |

**Returns**: `id`.

---

## `task_delete`

| Parameter | Type | Required |
|-----------|------|----------|
| `id` | `string` | yes |

Cascades to steps, runs, locks, checkpoints, and artifacts.

**Returns**: `id`, `deleted: true`.

---

## `task_start_run`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `task_id` | `string` | yes | |
| `step_id` | `string` | no | Step being executed |

Sets task `status` to `running`.

**Returns**: `run_id`.

---

## `task_end_run`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `run_id` | `string` | yes | From `task_start_run` |
| `status` | `string` | no | `done` / `failed` / `cancelled` (default: `done`) |
| `error` | `string` | no | Error message |
| `summary` | `string` | no | Execution summary |

Updates task status to match run outcome.

**Returns**: `run_id`.

---

## `task_append_step`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `task_id` | `string` | yes | |
| `title` | `string` | yes | Non-empty after trim |

Steps are auto-numbered by `seq` (1-based, incremented per task).

**Returns**: `step_id`.

---

## `task_update_step`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `step_id` | `string` | yes | From `task_append_step` |
| `status` | `string` | yes | `pending` / `running` / `done` / `failed` / `cancelled` |

**Returns**: `step_id`.

---

## `task_open_checkpoint`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `task_id` | `string` | yes | |
| `message` | `string` | yes | Message or question for the reviewer |
| `run_id` | `string` | no | |
| `risk_level` | `string` | no | `low` / `medium` / `high` / `critical` |

Sets task `status` to `waiting_checkpoint`.

**Returns**: `checkpoint_id`.

---

## `task_close_checkpoint`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `checkpoint_id` | `string` | yes | From `task_open_checkpoint` |
| `status` | `string` | no | `resolved` / `closed` (default: `closed`) |
| `task_status` | `string` | no | Task status after closing (default: `ready`) |

**Returns**: `checkpoint_id`.

---

## `task_acquire_lock`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `task_id` | `string` | yes | |
| `path` | `string` | yes | Canonical file path to lock |
| `run_id` | `string` | no | |
| `expires_at` | `string` | no | RFC3339 expiry time |

Fails with `TASK_LOCK_CONFLICT` if the path is already locked.

**Returns**: `lock_id`.

---

## `task_release_lock`

| Parameter | Type | Required |
|-----------|------|----------|
| `lock_id` | `string` | yes |

**Returns**: `lock_id`, `released: true`.

---

## `task_add_artifact`

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `task_id` | `string` | yes | |
| `kind` | `string` | yes | `file` / `summary` / `report` / `reference` (open string) |
| `run_id` | `string` | no | |
| `path` | `string` | no | File path |
| `content` | `string` | no | Inline content |

**Returns**: `artifact_id`.

---

## Status values

### Task status

| Value | Meaning |
|-------|---------|
| `backlog` | Not yet started |
| `ready` | Ready to run |
| `running` | Actively executing |
| `waiting_checkpoint` | Paused at a human-in-the-loop gate |
| `blocked` | Blocked (e.g. process restart, dependency) |
| `done` | Completed successfully |
| `failed` | Execution failed |
| `cancelled` | Cancelled |

### Step status

`pending` / `running` / `done` / `failed` / `cancelled`

### Run status

`running` / `done` / `failed` / `cancelled`

### Checkpoint status

`open` / `acknowledged` / `action_required` / `resolved` / `closed`

---

## Error codes

| Code | Meaning |
|------|---------|
| `INVALID_PATH` | Missing/invalid string args (`core::json`) |
| `TASK_INVALID_INPUT` | Empty title, etc. |
| `TASK_INVALID_STATUS` | Bad status value |
| `TASK_INVALID_KIND` | Bad `kind` value |
| `TASK_INVALID_OWNER` | Bad `owner` value |
| `TASK_INVALID_PRIORITY` | Bad `priority` value |
| `TASK_NOT_FOUND` | Unknown `id` |
| `TASK_LOCK_CONFLICT` | Path already locked by another task |
| `TASK_STORAGE_ERROR` | SQLite I/O, mutex error, etc. |
