# 版本控制（`git`）

中文 | [English](README.md)

[← 返回仓库说明](../../README.zh.md)

实现源码：[mod.rs](mod.rs)

## `git_status`

查看工作区文件变更状态。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 否 | 仓库路径，默认当前目录 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `changes` | `Change[]` | 变更列表 |
| `changes[].file` | `string` | 文件路径 |
| `changes[].status` | `"added" \| "modified" \| "deleted" \| "untracked"` | 变更类型 |

---

## `git_diff`

查看文件修改差异。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 否 | 仓库或文件路径，默认当前目录 |
| `staged` | `boolean` | 否 | 是否查看暂存区差异，默认 `false` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `diff` | `string` | diff 文本内容 |

---

## `git_commit`

暂存并提交变更。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `message` | `string` | 是 | 提交信息 |
| `files` | `string[]` | 否 | 指定暂存的文件列表，默认暂存全部变更 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `hash` | `string` | 提交的 commit hash |
| `message` | `string` | 提交信息 |

---

## `git_log`

查看提交历史记录。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 否 | 仓库或文件路径，默认当前目录 |
| `limit` | `number` | 否 | 返回条数，默认 10 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `commits` | `Commit[]` | 提交列表 |
| `commits[].hash` | `string` | commit hash |
| `commits[].message` | `string` | 提交信息 |
| `commits[].author` | `string` | 作者 |
| `commits[].date` | `string` | 提交时间（ISO 8601） |

## 错误码

本模块工具可能返回的 `error.code` 如下（定义见 [`error.rs`](error.rs)）。

| 错误码 | 说明 |
|--------|------|
| `GIT_ERROR` | 非 Git 仓库、命令执行失败、工作区状态不允许等 |

---

## `worktree_add`

创建新的关联工作树。若 `branch` 不存在则从 HEAD 新建该分支。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | `string` | 是 | 工作树名称 |
| `path` | `string` | 是 | 工作树目录路径 |
| `branch` | `string` | 否 | 关联分支；不存在时从 HEAD 新建 |
| `repo` | `string` | 否 | 仓库路径，默认 context root |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `string` | 工作树名称 |
| `path` | `string` | 工作树路径 |
| `branch` | `string` | 关联分支名 |

---

## `worktree_list`

列出所有关联工作树及其路径和锁定状态。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `repo` | `string` | 否 | 仓库路径，默认 context root |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `worktrees` | `Worktree[]` | 工作树列表 |
| `worktrees[].name` | `string` | 名称 |
| `worktrees[].path` | `string` | 路径 |
| `worktrees[].locked` | `boolean` | 是否已锁定 |

---

## `worktree_remove`

删除关联工作树。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | `string` | 是 | 工作树名称 |
| `force` | `boolean` | 否 | 强制删除（即使工作树已锁定），默认 `false` |
| `repo` | `string` | 否 | 仓库路径，默认 context root |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `string` | 已删除的工作树名称 |

---

## `worktree_lock`

锁定工作树，防止误删。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | `string` | 是 | 工作树名称 |
| `reason` | `string` | 否 | 锁定原因 |
| `repo` | `string` | 否 | 仓库路径，默认 context root |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `string` | 工作树名称 |
| `reason` | `string` | 锁定原因 |

---

## `worktree_unlock`

解锁已锁定的工作树。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | `string` | 是 | 工作树名称 |
| `repo` | `string` | 否 | 仓库路径，默认 context root |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `string` | 工作树名称 |
