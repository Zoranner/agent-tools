pub mod tool;

mod core;

pub use tool::{Tool, ToolError, ToolResult};

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "find")]
pub mod find;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "md")]
pub mod md;

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "exec")]
pub mod exec;

#[cfg(feature = "code")]
pub mod code;

#[cfg(feature = "office")]
pub mod office;

#[cfg(feature = "browser")]
pub mod browser;

#[cfg(feature = "design")]
pub mod design;

#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "todo")]
pub mod todo;

#[cfg(feature = "task")]
pub mod task;

#[cfg(feature = "interact")]
pub mod interact;
