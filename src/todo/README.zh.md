# 待办（`todo`）

中文 | [English](README.md)

[← 返回仓库说明](../../README.zh.md)

实现源码：[mod.rs](mod.rs)

待办数据保存在工作区内的 **JSON** 文件中（默认 **`.agent/todos.json`**），与 [`TodoContext`](mod.rs) 的根目录、沙箱语义一致（见库内 `core::path`）。每条待办有稳定 **`id`**（UUID v4），供更新与删除。

## `todo_add`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | `string` | 是 | 标题（去首尾空白后非空） |
| `description` | `string` | 否 | 详细说明 |
| `priority` | `string` | 否 | `low` / `medium` / `high` |
| `tags` | `string[]` | 否 | 标签；`todo_list` 可按单标签过滤 |

**返回**：`id`（新建项的 UUID 字符串）。新建项 **`status` 为 `pending`**。

---

## `todo_list`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `status` | `string` | 否 | `pending` / `done` / `cancelled` |
| `tag` | `string` | 否 | 项须包含该标签 |
| `limit` | `number` | 否 | 默认 50，最大 200 |

**排序**：先 **`pending`**，再 **`done`**，再 **`cancelled`**；同状态下 **`high` > `medium` > `low` > 无优先级**；再按 **`updated_at` 降序**。

**返回**：`items` 数组，元素为完整待办对象（`id`, `title`, `description`, `status`, `priority`, `tags`, `created_at`, `updated_at`）。

---

## `todo_update`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | `string` | 是 | 目标待办 |
| `title` | `string` | 否 | 新标题（若提供则非空） |
| `description` | `string` | 否 | 新说明 |
| `status` | `string` | 否 | `pending` / `done` / `cancelled` |
| `priority` | `string` / `null` | 否 | 新级别；**JSON `null` 清除优先级** |
| `tags` | `string[]` | 否 | 若提供则**整体替换**标签列表 |

未出现的字段保持不变。`updated_at` 自动更新。

---

## `todo_remove`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | `string` | 是 | 要删除的项 |

**返回**：`id`, `removed: true`。

## 错误码

| 错误码 | 说明 |
|--------|------|
| `INVALID_PATH` | 必填字符串参数缺失或类型错误（见 `core::json`） |
| `TODO_NOT_FOUND` | `id` 不存在（`todo_update` / `todo_remove`） |
| `TODO_INVALID_INPUT` | 标题为空等 |
| `TODO_INVALID_STATUS` | `status` 枚举非法 |
| `TODO_INVALID_PRIORITY` | `priority` 枚举非法 |
| `TODO_STORAGE_ERROR` | 读写 JSON、目录创建、文件损坏等 |
