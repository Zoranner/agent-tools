# 任务（`task`）

中文 | [English](README.md)

[← 返回仓库说明](../../README.zh.md)

实现源码：[mod.rs](mod.rs)

任务数据保存在工作区内的 **SQLite** 数据库中（默认 **`.agent/tasks.db`**）。每个实体有稳定 **`id`**（UUID v4）。启动时，遗留 `running` 状态的任务自动转为 `blocked`，过期的路径锁自动释放。

## 核心概念

| 实体 | 说明 |
|------|------|
| **Task** | 顶层工作单元。包含 `kind`、`status`、`owner`、可选的 `priority`、`tags`、`goal`、`acceptance`、`risk_level`。 |
| **TaskStep** | 任务内的有序子步骤，逐条追加与跟踪。 |
| **TaskRun** | 任务（或某步骤）的单次执行实例，记录开始/结束时间、状态、错误与摘要。 |
| **PathLock** | 文件路径的独占写锁，防止多任务并发写入同一文件。 |
| **Checkpoint** | 人工介入检查点。开启后任务转为 `waiting_checkpoint`；关闭后恢复执行。 |
| **Artifact** | 任务产出引用：文件路径、内联内容或外部引用。 |

## `task_create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | `string` | 是 | 标题（去首尾空白后非空） |
| `description` | `string` | 否 | 详细说明 |
| `kind` | `string` | 否 | `task` / `milestone` / `checkpoint`（默认 `task`） |
| `owner` | `string` | 否 | `agent` / `human`（默认 `agent`） |
| `priority` | `string` | 否 | `low` / `medium` / `high` |
| `tags` | `string[]` | 否 | 标签列表 |
| `goal` | `string` | 否 | 任务目标描述 |
| `acceptance` | `string` | 否 | 验收标准 |
| `risk_level` | `string` | 否 | `low` / `medium` / `high` / `critical` |

**返回**：`id`。新建任务 **`status` 为 `backlog`**。

---

## `task_list`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `status` | `string` | 否 | 按状态过滤 |
| `kind` | `string` | 否 | 按类型过滤 |
| `owner` | `string` | 否 | 按归属过滤 |
| `tag` | `string` | 否 | 项须包含该标签 |
| `limit` | `number` | 否 | 默认 50，最大 200 |

**排序**：按 `updated_at` 降序。

**返回**：`items` 数组，元素为完整任务对象。

---

## `task_get`

| 参数 | 类型 | 必填 |
|------|------|------|
| `id` | `string` | 是 |

**返回**：完整任务对象。

---

## `task_update`

Patch 语义——仅更新提供的字段。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | `string` | 是 | |
| `title` | `string` | 否 | 若提供则非空 |
| `description` | `string` | 否 | |
| `status` | `string` | 否 | 见下方状态值 |
| `priority` | `string` / `null` | 否 | **JSON `null` 清除优先级** |
| `blocked_reason` | `string` / `null` | 否 | JSON `null` 清除 |
| `last_error` | `string` / `null` | 否 | JSON `null` 清除 |
| `tags` | `string[]` | 否 | 若提供则**整体替换**标签列表 |
| `risk_level` | `string` / `null` | 否 | JSON `null` 清除 |

**返回**：`id`。

---

## `task_delete`

| 参数 | 类型 | 必填 |
|------|------|------|
| `id` | `string` | 是 |

级联删除关联的步骤、执行记录、路径锁、检查点与产出物。

**返回**：`id`, `deleted: true`。

---

## `task_start_run`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `task_id` | `string` | 是 | |
| `step_id` | `string` | 否 | 本次执行对应的步骤 |

将任务 `status` 设为 `running`。

**返回**：`run_id`。

---

## `task_end_run`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `run_id` | `string` | 是 | 来自 `task_start_run` |
| `status` | `string` | 否 | `done` / `failed` / `cancelled`（默认 `done`） |
| `error` | `string` | 否 | 错误信息 |
| `summary` | `string` | 否 | 执行摘要 |

根据执行结果同步更新任务状态。

**返回**：`run_id`。

---

## `task_append_step`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `task_id` | `string` | 是 | |
| `title` | `string` | 是 | 步骤标题（去首尾空白后非空） |

步骤按 `seq` 自动编号（从 1 开始，每个任务独立递增）。

**返回**：`step_id`。

---

## `task_update_step`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `step_id` | `string` | 是 | 来自 `task_append_step` |
| `status` | `string` | 是 | `pending` / `running` / `done` / `failed` / `cancelled` |

**返回**：`step_id`。

---

## `task_open_checkpoint`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `task_id` | `string` | 是 | |
| `message` | `string` | 是 | 给审阅者的消息或问题 |
| `run_id` | `string` | 否 | |
| `risk_level` | `string` | 否 | `low` / `medium` / `high` / `critical` |

将任务 `status` 设为 `waiting_checkpoint`。

**返回**：`checkpoint_id`。

---

## `task_close_checkpoint`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `checkpoint_id` | `string` | 是 | 来自 `task_open_checkpoint` |
| `status` | `string` | 否 | `resolved` / `closed`（默认 `closed`） |
| `task_status` | `string` | 否 | 关闭后任务的目标状态（默认 `ready`） |

**返回**：`checkpoint_id`。

---

## `task_acquire_lock`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `task_id` | `string` | 是 | |
| `path` | `string` | 是 | 要锁定的规范化文件路径 |
| `run_id` | `string` | 否 | |
| `expires_at` | `string` | 否 | RFC3339 格式的过期时间 |

若路径已被其他任务锁定，返回 `TASK_LOCK_CONFLICT`。

**返回**：`lock_id`。

---

## `task_release_lock`

| 参数 | 类型 | 必填 |
|------|------|------|
| `lock_id` | `string` | 是 |

**返回**：`lock_id`, `released: true`。

---

## `task_add_artifact`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `task_id` | `string` | 是 | |
| `kind` | `string` | 是 | `file` / `summary` / `report` / `reference`（开放字符串） |
| `run_id` | `string` | 否 | |
| `path` | `string` | 否 | 文件路径 |
| `content` | `string` | 否 | 内联内容 |

**返回**：`artifact_id`。

---

## 状态值

### 任务状态

| 值 | 含义 |
|----|------|
| `backlog` | 尚未开始 |
| `ready` | 准备就绪 |
| `running` | 正在执行 |
| `waiting_checkpoint` | 暂停于人工检查点 |
| `blocked` | 被阻塞（如进程重启、依赖未就绪） |
| `done` | 成功完成 |
| `failed` | 执行失败 |
| `cancelled` | 已取消 |

### 步骤状态

`pending` / `running` / `done` / `failed` / `cancelled`

### 执行状态

`running` / `done` / `failed` / `cancelled`

### 检查点状态

`open` / `acknowledged` / `action_required` / `resolved` / `closed`

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| `INVALID_PATH` | 必填字符串参数缺失或类型错误（见 `core::json`） |
| `TASK_INVALID_INPUT` | 标题为空等 |
| `TASK_INVALID_STATUS` | `status` 枚举非法 |
| `TASK_INVALID_KIND` | `kind` 枚举非法 |
| `TASK_INVALID_OWNER` | `owner` 枚举非法 |
| `TASK_INVALID_PRIORITY` | `priority` 枚举非法 |
| `TASK_NOT_FOUND` | `id` 不存在 |
| `TASK_LOCK_CONFLICT` | 路径已被其他任务锁定 |
| `TASK_STORAGE_ERROR` | SQLite 读写、互斥锁错误等 |
