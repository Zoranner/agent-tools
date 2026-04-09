# Agent Tools

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[中文](README.zh.md) | English

A Rust toolkit for AI agents: workspace file I/O, search, web fetch, Markdown helpers, Git, cross-session memory, pluggable human-in-the-loop I/O, persistent todos, and a SQLite-backed task store with steps, runs, locks, checkpoints, and artifacts. Each tool is described with JSON Schema and lines up with OpenAI Function Calling, Anthropic Tool Use, and similar formats.

## Quick start

Enable only what you need in `Cargo.toml`:

```toml
[dependencies]
agentool = { version = "0.1", features = ["fs", "find", "web", "git"] }
```

All published features:

```toml
agentool = { version = "0.1", features = ["full"] }
```

Heavy crates (`tokio`, `reqwest`, `git2`, `walkdir`, `regex`, `chrono`, …) are **optional**: each feature in `Cargo.toml` pulls only what it needs. The always-on footprint is `async-trait`, `serde_json`, and `thiserror` (plus the `Tool` trait when you enable no features).

**Example: write and read a file**

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

## Features

| Feature | Tools |
|---------|--------|
| `fs` | `read_file` / `write_file` / `edit_file` / `create_directory` / `list_directory` / `delete_file` / `move_file` / `copy_file` |
| `find` | `grep_search` / `glob_search` |
| `web` | `web_search` / `web_fetch` |
| `md` | `extract_toc` / `markdown_stats` |
| `git` | `git_status` / `git_diff` / `git_commit` / `git_log` |
| `memory` | `memory_write` / `memory_update` / `memory_read` / `memory_search` |
| `interact` | `interact_ask` / `interact_confirm` / `interact_notify` |
| `todo` | `todo_add` / `todo_list` / `todo_update` / `todo_remove` |
| `task` | `task_create` / `task_list` / `task_get` / `task_update` / `task_delete` / `task_start_run` / `task_end_run` / `task_append_step` / `task_update_step` / `task_open_checkpoint` / `task_close_checkpoint` / `task_acquire_lock` / `task_release_lock` / `task_add_artifact` |
| `full` | All published modules |

> `exec` / `code` / `office` / `browser` / `design` / `gui` are planned but not shipped yet.

## Tool reference

Each feature has a **`README.md`** (English) next to `mod.rs` under `src/<feature>/`. Chinese versions live in **`README.zh.md`** in the same folder.

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
| `task` | [src/task/README.md](src/task/README.md) | [src/task/README.zh.md](src/task/README.zh.md) |

Placeholder modules: [src/exec/README.md](src/exec/README.md) · [src/code/README.md](src/code/README.md) · [src/office/README.md](src/office/README.md) · [src/browser/README.md](src/browser/README.md) · [src/design/README.md](src/design/README.md) · [src/gui/README.md](src/gui/README.md) (each has a `README.zh.md`).

## Documentation hub

- [docs/README.md](docs/README.md) — how docs are organized, conventions, links  
- [docs/README.zh.md](docs/README.zh.md) — 中文索引

## Response shape

All tools return JSON in a single envelope.

**Success**

```json
{
  "success": true,
  "data": {}
}
```

**Failure**

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message"
  }
}
```

In Rust, `Tool::execute` returns `Result<serde_json::Value, ToolError>`: on success the `Ok` value already includes the `success` / `data` shell; hosts map `Err(ToolError)` to `success: false`.

## License

[MIT](LICENSE)
