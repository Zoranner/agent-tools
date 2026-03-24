pub mod error;
pub mod tool;

pub use error::{ToolError, ToolErrorCode};
pub use tool::Tool;

pub type ToolResult = Result<serde_json::Value, ToolError>;
