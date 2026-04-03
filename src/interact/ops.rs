use serde_json::{json, Value};

use crate::core::json::{json_str, ok_data};
use crate::tool::ToolResult;

use super::backend::NotifyLevel;
use super::InteractContext;

fn options_from_params(params: &Value) -> Option<Vec<String>> {
    params.get("options").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect::<Vec<_>>()
    })
}

pub(crate) async fn op_ask(ctx: &InteractContext, params: &Value) -> ToolResult {
    let question = json_str(params, "question")?;
    let mut options = options_from_params(params);
    if let Some(ref o) = options {
        if o.is_empty() {
            options = None;
        }
    }
    let timeout_secs = params.get("timeout").and_then(|v| v.as_u64());
    let answer = ctx.backend.ask(question, options, timeout_secs).await?;
    Ok(ok_data(json!({ "answer": answer })))
}

pub(crate) async fn op_confirm(ctx: &InteractContext, params: &Value) -> ToolResult {
    let message = json_str(params, "message")?;
    let default = params
        .get("default")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let timeout_secs = params.get("timeout").and_then(|v| v.as_u64());
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
