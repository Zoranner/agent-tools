# Git (`git`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs)

## `git_status`

Working tree status.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `path` | `string` | no | Repository path; defaults to the context root, then discovers the nearest enclosing Git repository |

**Returns**

| Field | Type |
|-------|------|
| `changes` | `Change[]` |
| `changes[].file` | `string` |
| `changes[].status` | `"added" \| "modified" \| "deleted" \| "untracked"` |

---

## `git_diff`

Show a textual diff.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `path` | `string` | no | Repo or file path; defaults to the context root, then discovers the nearest enclosing Git repository |
| `staged` | `boolean` | no | Staged diff; default `false` |

**Returns**

| Field | Type |
|-------|------|
| `diff` | `string` |

---

## `git_commit`

Stage and commit.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `message` | `string` | yes | Commit message |
| `path` | `string` | no | Repository path; defaults to the context root, then discovers the nearest enclosing Git repository |
| `files` | `string[]` | no | Stage only these paths; default all changes |

**Returns**

| Field | Type |
|-------|------|
| `hash` | `string` |
| `message` | `string` |

---

## `git_log`

Commit history.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `path` | `string` | no | Repo or file path; defaults to the context root, then discovers the nearest enclosing Git repository |
| `limit` | `number` | no | Default 10 |

**Returns**

| Field | Type |
|-------|------|
| `commits` | `Commit[]` |
| `commits[].hash` | `string` |
| `commits[].message` | `string` |
| `commits[].author` | `string` |
| `commits[].date` | `string` | ISO 8601 |

## Error codes

See [`error.rs`](error.rs).

| Code | Meaning |
|------|---------|
| `GIT_ERROR` | Not a repo, libgit2 failure, invalid state, etc. |

---

## `worktree_add`

Create a new linked worktree. If `branch` does not exist it is created from HEAD.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `name` | `string` | yes | Worktree name |
| `path` | `string` | yes | Directory path for the new worktree |
| `branch` | `string` | no | Branch to check out; created from HEAD if absent |
| `repo` | `string` | no | Repository path (default: context root) |

**Returns**

| Field | Type |
|-------|------|
| `name` | `string` |
| `path` | `string` |
| `branch` | `string` |

---

## `worktree_list`

List all linked worktrees with their path and lock status.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `repo` | `string` | no | Repository path (default: context root) |

**Returns**

| Field | Type |
|-------|------|
| `worktrees` | `Worktree[]` |
| `worktrees[].name` | `string` |
| `worktrees[].path` | `string` |
| `worktrees[].locked` | `boolean` |

---

## `worktree_remove`

Remove a linked worktree.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `name` | `string` | yes | Worktree name |
| `force` | `boolean` | no | Force removal even if the worktree is locked (default `false`) |
| `repo` | `string` | no | Repository path (default: context root) |

**Returns**

| Field | Type |
|-------|------|
| `name` | `string` |

---

## `worktree_lock`

Lock a worktree to prevent accidental removal.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `name` | `string` | yes | Worktree name |
| `reason` | `string` | no | Lock reason |
| `repo` | `string` | no | Repository path (default: context root) |

**Returns**

| Field | Type |
|-------|------|
| `name` | `string` |
| `reason` | `string` |

---

## `worktree_unlock`

Unlock a previously locked worktree.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `name` | `string` | yes | Worktree name |
| `repo` | `string` | no | Repository path (default: context root) |

**Returns**

| Field | Type |
|-------|------|
| `name` | `string` |
