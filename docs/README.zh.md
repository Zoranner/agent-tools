# agentool 文档

中文 | [English](README.md)

本目录汇总 **agentool** crate 的人工文档索引。类型与 trait 的权威说明仍以源码与 `rustdoc` 为准（`cargo doc --all-features --no-deps --open`）。

## 文档放在哪

| 内容 | 位置 |
|------|------|
| 仓库总览与快速开始 | [README.md](../README.md)（英文）· [README.zh.md](../README.zh.md)（中文） |
| 各 feature 工具说明（参数、返回字段、错误码） | `src/<feature>/README.md`（英文）· 同目录 `README.zh.md`（中文） |
| Rust 类型定义 | 源码与 `rustdoc` |

## 约定

- **Cargo feature** 与可选依赖一一对应（见仓库根目录 `Cargo.toml`）：只开用到的工具模块可缩短编译时间与传递依赖。
- 各层 **`README.md` 默认为英文**。
- **中文** 使用同目录 **`README.zh.md`**，英文页顶部提供切换链接。
- 错误码 `code` 为稳定约定；具体枚举与映射见各模块 `error.rs` 及对应 README 表格。

## 阅读路径

**刚接触本库**

1. [README.zh.md](../README.zh.md) — 安装、feature、统一返回结构  
2. 从功能表进入 `src/<feature>/README.zh.md`（或英文 `README.md`）  
3. 按该模块文档连接 `FsContext`、`WebContext` 等上下文  

**查某个工具**

- 以 feature 的 README 为准（JSON 字段名、类型、错误码）。  
- 工具名为小写加下划线（如 `grep_search`、`memory_write`）。

## 发版

推送 `v*` 标签后 CI 会执行 `fmt`、`clippy`、测试，通过后 `cargo publish`。见 [.github/workflows/cargo-publish.yml](../.github/workflows/cargo-publish.yml)。
