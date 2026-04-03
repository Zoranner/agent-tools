//! 库内部通用逻辑：供多个工具 feature 复用，**不**作为对外 API 暴露。
//!
//! - [`json`]：成功响应外壳、通用 `string` 参数解析
//! - [`blocking`]：`spawn_blocking` 包装
//! - [`path`]：工作区根路径与沙箱（仅在有文件类/路径类 feature 时编译）

pub(crate) mod blocking;
pub(crate) mod json;

#[cfg(any(
    feature = "fs",
    feature = "md",
    feature = "git",
    feature = "find",
    feature = "memory"
))]
pub(crate) mod path;
