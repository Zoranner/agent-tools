# Agent Tools

面向 AI 智能体的 Rust 工具库，为 LLM 驱动的应用提供文件系统操作、内容搜索、网络获取、文档分析、版本控制和跨会话记忆能力。每个工具通过 JSON Schema 描述，与 OpenAI Function Calling / Anthropic Tool Use 等主流格式直接兼容。

## 快速开始

在 `Cargo.toml` 中按需添加依赖：

```toml
[dependencies]
agentool = { version = "0.1", features = ["fs", "search", "web", "git"] }
```

开启全部已发布功能：

```toml
agentool = { version = "0.1", features = ["full"] }
```

**示例：写入并读取文件**

```rust
use std::sync::Arc;

use agentool::Tool;
use agentool::fs::{all_tools, FsContext};

async fn example() -> Result<(), agentool::ToolError> {
    let ctx = Arc::new(FsContext::new(None, false).expect("workspace root"));
    let tools = all_tools(ctx);

    let write = tools.iter().find(|t| t.name() == "write_file").unwrap();
    let read  = tools.iter().find(|t| t.name() == "read_file").unwrap();

    write.execute(serde_json::json!({
        "path": "example.txt",
        "content": "hello\n",
    })).await?;

    let out = read.execute(serde_json::json!({ "path": "example.txt" })).await?;
    println!("{}", out["data"]["content"]);
    Ok(())
}
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
| `interact` | `ask` / `confirm` / `notify` |
| `full` | 全部已发布模块 |

> `exec` / `code` / `office` / `browser` / `design` / `gui` / `todo` 等模块尚在规划中，暂未发布。

## 工具参考

### 文件系统（`fs`）

> **路径与沙箱**
>
> 所有 `fs` 工具共享同一个 `FsContext`：
>
> | 模式 | 说明 |
> |------|------|
> | `FsContext::new(root, false)`（默认） | **沙箱模式**：路径解析后必须落在工作区根下（含根本身），否则返回 `INVALID_PATH` |
> | `FsContext::new(root, true)` | **放宽模式**：相对路径基于进程当前目录解析，不做范围限制 |
>
> `root` 为 `None` 时取进程当前目录。路径传入前先做 `.` / `..` 语法归一化；绝对路径若落在根外同样被拒绝。

---

#### `read_file`

读取文本文件内容，支持按行分页。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径（须为普通文件） |
| `offset` | `integer` | 否 | 起始行号（从 1 开始，须 ≥ 1） |
| `limit` | `integer` | 否 | 读取行数上限（须为非负整数） |

`offset` 和 `limit` 只接受 JSON 整数，不接受浮点数。

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `content` | `string` | 文件内容，保留原始换行符（CRLF 不归一化）及末尾换行 |
| `total_lines` | `number` | 文件逻辑行数 |

---

#### `write_file`

写入文件；文件不存在时创建，已存在则覆盖。自动递归创建不存在的父目录。

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

精确替换文件中的某段文本。`old_text` 必须在文件中**唯一**出现，且不得为空字符串。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |
| `old_text` | `string` | 是 | 待替换的原始文本（须唯一） |
| `new_text` | `string` | 是 | 替换后的新文本 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 文件的绝对路径 |

---

#### `create_directory`

创建目录，支持递归创建多级目录。目录已存在时不报错。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 目录路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 目录的绝对路径 |

---

#### `list_directory`

列出目录中的文件和子目录，结果按名称升序排序。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 目录路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `entries` | `Entry[]` | 条目列表（按 `name` 升序） |
| `entries[].name` | `string` | 文件或目录名 |
| `entries[].type` | `"file" \| "directory"` | 类型 |
| `entries[].size` | `number` | 文件大小（字节），目录为 0 |

---

#### `delete_file`

删除普通文件。若目标是目录，返回 `INVALID_PATH`。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 被删除文件的绝对路径 |

---

#### `move_file`

移动或重命名普通文件。目标路径已存在时失败，自动创建目标路径的父目录。

跨卷移动时会自动回退为「复制 + 删除源」；若删除源失败，会尝试清除已创建的目标副本后返回错误，不留无主文件。

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

复制普通文件。目标路径已存在时失败，自动创建目标路径的父目录。

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

### 搜索（`search`）

#### `grep_search`

按关键词或正则表达式搜索文件内容。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | 搜索关键词或正则表达式 |
| `path` | `string` | 否 | 搜索根目录，默认当前目录 |
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

按文件名 Glob 模式匹配文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | Glob 模式，如 `**/*.md` |
| `path` | `string` | 否 | 搜索根目录，默认当前目录 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `files` | `string[]` | 匹配的文件路径列表 |

---

### 网络（`web`）

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

抓取指定网页并转换为 Markdown。

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

### 文档（`md`）

#### `extract_toc`

提取 Markdown 文档的目录结构。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `toc` | `TocItem[]` | 目录列表 |
| `toc[].level` | `number` | 标题层级（1–6） |
| `toc[].title` | `string` | 标题文本 |
| `toc[].line` | `number` | 所在行号 |

---

#### `count_words`

统计文档字数及结构信息。

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

### 版本控制（`git`）

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

### 记忆（`memory`）

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

### 交互（`interact`）

#### `ask`

向用户提问，等待并返回回答。

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

向用户请求确认，等待是/否结果。

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

向用户发送通知，无需等待回复。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `message` | `string` | 是 | 通知内容 |
| `level` | `"info" \| "warning" \| "error"` | 否 | 通知级别，默认 `"info"` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `sent` | `boolean` | 是否发送成功 |

---

## 接口规范

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

在 Rust 中，`Tool::execute` 返回 `Result<serde_json::Value, ToolError>`：成功时 `Ok` 值已包含 `success / data` 外壳；失败时由宿主/runtime 将 `Err(ToolError)` 映射为 `success: false` 的 JSON。

### 错误码

| 错误码 | 说明 |
|--------|------|
| `FILE_NOT_FOUND` | 文件或目录不存在 |
| `PERMISSION_DENIED` | 无读写权限 |
| `FILE_ALREADY_EXISTS` | 目标文件已存在 |
| `DIRECTORY_NOT_EMPTY` | 目录非空，无法删除 |
| `PATTERN_NOT_FOUND` | `edit_file` 的 `old_text` 未找到 |
| `PATTERN_NOT_UNIQUE` | `edit_file` 的 `old_text` 匹配到多处 |
| `INVALID_PATH` | 路径格式不合法或超出沙箱范围 |
| `NETWORK_ERROR` | 网络请求失败 |
| `GIT_ERROR` | Git 操作失败 |
| `MEMORY_KEY_NOT_FOUND` | 记忆条目不存在 |
