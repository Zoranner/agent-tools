//! 库内部通用逻辑：供多个工具 feature 复用，**不**作为对外 API 暴露。
//!
//! 子模块按职责划分（例如 [`path`] 负责工作区根目录下的路径解析与沙箱约束）。
//!
//! 整个 `core` 由根 `lib.rs` 按 feature 条件编译，避免无依赖 feature 时编进空壳。

pub(crate) mod path;
