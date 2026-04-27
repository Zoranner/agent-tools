use serde_json::{json, Value};

use crate::core::json::{json_bool_opt, json_str, json_string_array_opt, json_u64_opt, ok_data};
use crate::tool::ToolResult;

use super::backend::NotifyLevel;
use super::InteractContext;

fn options_from_params(params: &Value) -> Result<Option<Vec<String>>, crate::tool::ToolError> {
    json_string_array_opt(params, "options")
}

pub(crate) async fn op_ask(ctx: &InteractContext, params: &Value) -> ToolResult {
    let question = json_str(params, "question")?;
    let mut options = options_from_params(params)?;
    if let Some(ref o) = options {
        if o.is_empty() {
            options = None;
        }
    }
    let timeout_secs = json_u64_opt(params, "timeout")?;
    let answer = ctx.backend.ask(question, options, timeout_secs).await?;
    Ok(ok_data(json!({ "answer": answer })))
}

pub(crate) async fn op_confirm(ctx: &InteractContext, params: &Value) -> ToolResult {
    let message = json_str(params, "message")?;
    let default = json_bool_opt(params, "default")?.unwrap_or(false);
    let timeout_secs = json_u64_opt(params, "timeout")?;
    let confirmed = ctx.backend.confirm(message, default, timeout_secs).await?;
    Ok(ok_data(json!({ "confirmed": confirmed })))
}

pub(crate) async fn op_notify(ctx: &InteractContext, params: &Value) -> ToolResult {
    let message = json_str(params, "message")?;
    let level = match params.get("level").and_then(|v| v.as_str()) {
        None => NotifyLevel::Info,
        Some(s) => NotifyLevel::parse(s)?,
    };
    let sent = ctx.backend.notify(message, level).await?;
    Ok(ok_data(json!({ "sent": sent })))
}
