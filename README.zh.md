# Agent Tools

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

中文 | [English](README.md)

面向 AI 智能体的 Rust 工具库，为 LLM 驱动的应用提供文件系统操作、工作区查找、网络获取、文档分析、版本控制、跨会话记忆、可插拔人机交互与工作区待办持久化能力。每个工具通过 JSON Schema 描述，与 OpenAI Function Calling / Anthropic Tool Use 等主流格式直接兼容。

## 快速开始

在 `Cargo.toml` 中按需添加依赖：

```toml
[dependencies]
agentool = { version = "0.1", features = ["fs", "find", "web", "git"] }
```

开启全部已发布功能：

```toml
agentool = { version = "0.1", features = ["full"] }
```

`tokio`、`reqwest`、`git2`、`walkdir`、`regex`、`chrono` 等较重依赖均为 **可选**，由 `Cargo.toml` 里对应 feature 按需启用；不启用任何工具 feature 时，常驻依赖主要是 `async-trait`、`serde_json`、`thiserror` 与核心 `Tool` trait。

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
| `find` | `grep_search` / `glob_search` |
| `web` | `web_search` / `web_fetch` |
| `md` | `extract_toc` / `markdown_stats` |
| `git` | `git_status` / `git_diff` / `git_commit` / `git_log` |
| `memory` | `memory_write` / `memory_update` / `memory_read` / `memory_search` |
| `interact` | `interact_ask` / `interact_confirm` / `interact_notify` |
| `todo` | `todo_add` / `todo_list` / `todo_update` / `todo_remove` |
| `full` | 全部已发布模块 |

> `exec` / `code` / `office` / `browser` / `design` / `gui` 等模块尚在规划中，暂未发布。

## 工具参考

每个 feature 在 `src/<feature>/` 与 `mod.rs` 同目录提供 **`README.md`（英文）** 与 **`README.zh.md`（中文）**，便于对照实现。

| Feature | English | 中文 |
|---------|---------|------|
| `fs` | [src/fs/README.md](src/fs/README.md) | [src/fs/README.zh.md](src/fs/README.zh.md) |
| `find` | [src/find/README.md](src/find/README.md) | [src/find/README.zh.md](src/find/README.zh.md) |
| `web` | [src/web/README.md](src/web/README.md) | [src/web/README.zh.md](src/web/README.zh.md) |
| `md` | [src/md/README.md](src/md/README.md) | [src/md/README.zh.md](src/md/README.zh.md) |
| `git` | [src/git/README.md](src/git/README.md) | [src/git/README.zh.md](src/git/README.zh.md) |
| `memory` | [src/memory/README.md](src/memory/README.md) | [src/memory/README.zh.md](src/memory/README.zh.md) |
| `interact` | [src/interact/README.md](src/interact/README.md) | [src/interact/README.zh.md](src/interact/README.zh.md) |
| `todo` | [src/todo/README.md](src/todo/README.md) | [src/todo/README.zh.md](src/todo/README.zh.md) |

规划中模块见 [src/exec/README.zh.md](src/exec/README.zh.md) 等同目录中文说明（与英文 README 成对）。

## 文档中心

- [docs/README.md](docs/README.md) — English index  
- [docs/README.zh.md](docs/README.zh.md) — 文档结构与约定（中文）

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

## 许可证

[MIT](LICENSE)
