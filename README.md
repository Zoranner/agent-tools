# Agent Tools

面向文档编辑智能体的工具套件，提供文件系统操作、内容搜索、网络获取、文档分析、版本控制和跨会话记忆能力。

## 快速开始

在 `Cargo.toml` 中添加依赖，通过 feature 按需引入功能模块：

```toml
[dependencies]
agentool = { version = "0.1", features = ["fs", "search", "web", "git"] }
```

开启全部功能：

```toml
agentool = { version = "0.1", features = ["full"] }
```

## 功能模块

| Feature | 工具 |
|---------|------|
| `fs` | `read_file` / `write_file` / `edit_file` / `create_directory` / `list_directory` / `delete_file` / `move_file` / `copy_file` |
| `search` | `grep_search` / `glob_search` |
| `web` | `web_search` / `web_fetch` |
| `md` | `extract_toc` / `count_words` |
| `git` | `git_status` / `git_diff` / `git_commit` / `git_log` |
| `memory` | `memory_write` / `memory_read` / `memory_search` |
| `exec` | 执行类工具 |
| `code` | 代码分析工具 |
| `office` | Office 文档工具 |
| `browser` | 浏览器操作工具 |
| `design` | 设计稿工具 |
| `gui` | GUI 交互工具 |
| `todo` | 任务管理工具 |
| `interact` | `ask` / `confirm` / `notify` |
| `full` | 全部模块 |

## 接口规范

### 工具定义格式

每个工具使用 JSON Schema 描述，兼容 OpenAI Function Calling 格式，可直接适配主流 LLM 供应商（Anthropic、OpenAI、Google 等）。

```json
{
  "name": "tool_name",
  "description": "工具功能描述",
  "parameters": {
    "type": "object",
    "properties": {
      "param1": {
        "type": "string",
        "description": "参数说明"
      },
      "param2": {
        "type": "number",
        "description": "参数说明"
      }
    },
    "required": ["param1"]
  }
}
```

### 返回值格式

所有工具返回统一的 JSON 结构。

**成功**

```json
{
  "success": true,
  "data": {}
}
```

**失败**

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述"
  }
}
```

### 错误码

| 错误码 | 说明 |
|--------|------|
| `FILE_NOT_FOUND` | 文件或目录不存在 |
| `PERMISSION_DENIED` | 无读写权限 |
| `FILE_ALREADY_EXISTS` | 目标文件已存在 |
| `DIRECTORY_NOT_EMPTY` | 目录非空，无法删除 |
| `PATTERN_NOT_UNIQUE` | `edit_file` 的 `old_text` 在文件中匹配到多处 |
| `PATTERN_NOT_FOUND` | `edit_file` 的 `old_text` 未找到 |
| `INVALID_PATH` | 路径格式不合法 |
| `NETWORK_ERROR` | 网络请求失败 |
| `GIT_ERROR` | Git 操作失败 |
| `MEMORY_KEY_NOT_FOUND` | 记忆条目不存在 |

---

### `Tool::execute` 与 JSON 外壳

库内 `Tool::execute` 的返回类型为 `Result<serde_json::Value, ToolError>`。**成功**时，`Ok` 中的 JSON 已包含上文约定的 `success` / `data` 外壳；**失败**时为 `Err(ToolError)`，由宿主/runtime 映射为 `success: false` 与 `error` 对象（字段与错误码表一致）。

### 文件系统路径与安全（`fs`）

使用 `agentool::fs::FsContext` 为全部 `fs` 工具提供共享配置：

| 项 | 说明 |
|----|------|
| 工作区根 | `FsContext::new(root, allow_outside_root)` 中 `root` 为 `None` 时，取进程当前工作目录并规范化为绝对路径；否则使用给定目录作为根 |
| `allow_outside_root == false`（默认） | 沙箱模式：解析后的路径必须落在工作区根之下（含边界），否则 `INVALID_PATH` |
| `allow_outside_root == true` | 放宽模式：相对路径相对进程当前工作目录解析，不做「必须在根下」的校验 |

路径会先进行 `.` / `..` 的语法归一化。沙箱模式下，若传入绝对路径且落在根外，同样返回 `INVALID_PATH`。

**各工具补充语义**

- `read_file`：目标必须是**普通文件**；若提供 `offset`，须 ≥ 1。`offset` 与 `limit` 须为**非负 JSON 整数**（不接受浮点数）。返回内容**保留原始换行符**（CRLF 不被归一化为 LF），也保留文件末尾换行；`total_lines` 为逻辑行数（与 `str::lines` 计数一致）。
- `write_file`：覆盖已存在文件；**自动创建不存在的父目录**。
- `edit_file`：`old_text` 不得为空；匹配次数为字面量子串匹配（与 `str::match_indices` 计数一致）。
- `list_directory`：返回条目按 `name` 升序排序，便于调用方做稳定比较与回放。
- `delete_file`：仅删除**普通文件**；若目标是目录，返回 `INVALID_PATH`（请使用其他流程处理目录删除）。
- `move_file` / `copy_file`：源必须是普通文件；若目标路径已存在（任意类型），返回 `FILE_ALREADY_EXISTS`；两者都会自动创建目标路径不存在的父目录。`move_file` 在 `rename` 失败时会尝试「复制 + 删除源」（用于跨卷等场景）；若删除源失败，会 best-effort 清除已创建的目标副本，再返回错误，以避免留下无主文件。

**Rust 示例**

```rust
use std::sync::Arc;

use agentool::Tool;
use agentool::fs::{all_tools, FsContext};

async fn example_fs_write_and_read() -> Result<(), agentool::ToolError> {
    let ctx = Arc::new(FsContext::new(None, false).expect("workspace root"));
    let tools = all_tools(ctx);

    let write = tools
        .iter()
        .find(|t| t.name() == "write_file")
        .expect("write_file tool");
    let read = tools
        .iter()
        .find(|t| t.name() == "read_file")
        .expect("read_file tool");

    write
        .execute(serde_json::json!({
            "path": "example.txt",
            "content": "hello\n",
        }))
        .await?;

    let out = read
        .execute(serde_json::json!({ "path": "example.txt" }))
        .await?;
    assert_eq!(out["success"], true);
    Ok(())
}
```

## 工具列表

### 文件系统

#### `read_file`

读取文件内容（仅普通文件）。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |
| `offset` | `integer` | 否 | 起始行号（从 1 开始，若提供则必须 ≥ 1） |
| `limit` | `integer` | 否 | 读取行数（必须为非负整数） |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `content` | `string` | 文件内容（保留原始换行符，含末尾换行） |
| `total_lines` | `number` | 文件逻辑行数 |

---

#### `write_file`

写入文件；文件不存在时创建，**并递归创建不存在的父目录**；已存在则覆盖。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |
| `content` | `string` | 是 | 写入内容 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 文件的绝对路径 |

---

#### `edit_file`

精确替换文件中的某段文本，要求 `old_text` 在文件中唯一；`old_text` 不得为空字符串。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |
| `old_text` | `string` | 是 | 待替换的原始文本 |
| `new_text` | `string` | 是 | 替换后的新文本 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 文件的绝对路径 |

---

#### `create_directory`

创建目录，支持递归创建多级目录。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 目录路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 目录的绝对路径 |

---

#### `list_directory`

列出目录内容；返回条目按 `name` 升序排序。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 目录路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `entries` | `Entry[]` | 条目列表 |
| `entries[].name` | `string` | 文件或目录名 |
| `entries[].type` | `"file" \| "directory"` | 类型 |
| `entries[].size` | `number` | 文件大小（字节），目录为 0 |

---

#### `delete_file`

删除**普通文件**（非目录）。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 被删除文件的绝对路径 |

---

#### `move_file`

移动或重命名普通文件；目标路径已存在时失败。会自动创建目标路径不存在的父目录。若系统 `rename` 不可用，可能回退为复制后删除源文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `source` | `string` | 是 | 源文件路径 |
| `destination` | `string` | 是 | 目标路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `source` | `string` | 源文件的绝对路径 |
| `destination` | `string` | 目标文件的绝对路径 |

---

#### `copy_file`

复制普通文件；目标路径已存在时失败。会自动创建目标路径不存在的父目录。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `source` | `string` | 是 | 源文件路径 |
| `destination` | `string` | 是 | 目标路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `source` | `string` | 源文件的绝对路径 |
| `destination` | `string` | 目标文件的绝对路径 |

---

### 搜索

#### `grep_search`

按关键词或正则表达式搜索文件内容。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | 搜索关键词或正则表达式 |
| `path` | `string` | 否 | 搜索范围，默认当前目录 |
| `glob` | `string` | 否 | 文件名过滤，如 `**/*.md` |
| `ignore_case` | `boolean` | 否 | 是否忽略大小写，默认 `false` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `matches` | `Match[]` | 匹配列表 |
| `matches[].file` | `string` | 文件路径 |
| `matches[].line` | `number` | 行号 |
| `matches[].content` | `string` | 匹配行内容 |

---

#### `glob_search`

按文件名模式匹配文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | Glob 模式，如 `**/*.md` |
| `path` | `string` | 否 | 搜索根目录，默认当前目录 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `files` | `string[]` | 匹配的文件路径列表 |

---

### 网络

#### `web_search`

搜索网络，返回相关资料摘要。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 搜索关键词 |
| `limit` | `number` | 否 | 返回结果数量，默认 5 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `Result[]` | 搜索结果列表 |
| `results[].title` | `string` | 页面标题 |
| `results[].url` | `string` | 页面 URL |
| `results[].snippet` | `string` | 内容摘要 |

---

#### `web_fetch`

抓取指定网页内容并转为 Markdown。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `url` | `string` | 是 | 网页 URL |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `content` | `string` | 转换后的 Markdown 内容 |
| `title` | `string` | 页面标题 |
| `url` | `string` | 实际访问的 URL（含重定向） |

---

### 文档

#### `extract_toc`

提取文档的目录结构。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `toc` | `TocItem[]` | 目录列表 |
| `toc[].level` | `number` | 标题层级（1-6） |
| `toc[].title` | `string` | 标题文本 |
| `toc[].line` | `number` | 所在行号 |

---

#### `count_words`

统计文档字数、段落数、标题数。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `characters` | `number` | 字符数（不含空格） |
| `words` | `number` | 词数 |
| `paragraphs` | `number` | 段落数 |
| `headings` | `number` | 标题数 |
| `lines` | `number` | 总行数 |

---

### 版本控制

#### `git_status`

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

#### `git_diff`

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

#### `git_commit`

暂存并提交文档变更。

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

#### `git_log`

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

---

### 记忆

#### `memory_write`

存储一条记忆条目。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 记忆标识符 |
| `content` | `string` | 是 | 记忆内容 |
| `tags` | `string[]` | 否 | 标签，用于分类和搜索 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 记忆标识符 |

---

#### `memory_read`

读取指定记忆条目。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 记忆标识符 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 记忆标识符 |
| `content` | `string` | 记忆内容 |
| `tags` | `string[]` | 标签列表 |
| `created_at` | `string` | 创建时间（ISO 8601） |
| `updated_at` | `string` | 更新时间（ISO 8601） |

---

#### `memory_search`

按关键词搜索历史记忆。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 搜索关键词 |
| `tags` | `string[]` | 否 | 按标签过滤 |
| `limit` | `number` | 否 | 返回数量，默认 10 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `MemoryItem[]` | 匹配的记忆列表 |
| `results[].key` | `string` | 记忆标识符 |
| `results[].content` | `string` | 记忆内容 |
| `results[].tags` | `string[]` | 标签列表 |

---

### 交互

#### `ask`

向用户或其他智能体提问，等待并返回回答。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `question` | `string` | 是 | 问题内容 |
| `options` | `string[]` | 否 | 候选选项，提供时为单选，不提供时为自由回答 |
| `timeout` | `number` | 否 | 等待超时秒数，超时返回错误 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `answer` | `string` | 回答内容 |

---

#### `confirm`

向用户或其他智能体请求确认，等待并返回是/否结果。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `message` | `string` | 是 | 确认提示内容 |
| `default` | `boolean` | 否 | 超时或无响应时的默认值，默认 `false` |
| `timeout` | `number` | 否 | 等待超时秒数 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `confirmed` | `boolean` | 是否确认 |

---

#### `notify`

向用户或其他智能体发送通知，无需等待回复。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `message` | `string` | 是 | 通知内容 |
| `level` | `"info" \| "warning" \| "error"` | 否 | 通知级别，默认 `"info"` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `sent` | `boolean` | 是否发送成功 |
