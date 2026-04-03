# Git (`git`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs)

## `git_status`

Working tree status.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `path` | `string` | no | Repository path; default current directory |

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
| `path` | `string` | no | Repo or file path; default cwd |
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
| `path` | `string` | no | Repo or file path; default cwd |
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
