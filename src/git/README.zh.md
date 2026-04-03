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
