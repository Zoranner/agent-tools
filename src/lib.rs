pub mod error;
pub mod tool;

pub use error::{ToolError, ToolErrorCode};
pub use tool::Tool;

pub type ToolResult = Result<serde_json::Value, ToolError>;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "search")]
pub mod search;

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

#[cfg(feature = "interact")]
pub mod interact;
