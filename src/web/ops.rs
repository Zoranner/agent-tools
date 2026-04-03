use reqwest::Url;
use serde_json::{json, Value};

use crate::tool::{ToolError, ToolResult};

use super::error::{tool_error, WebErrorCode};
use super::WebContext;

fn ok_data(data: Value) -> Value {
    json!({
        "success": true,
        "data": data,
    })
}

fn json_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
        tool_error(
            WebErrorCode::NetworkError,
            format!("missing or invalid `{key}`"),
        )
    })
}

/// Non-negative limit: JSON integer or whole-number float, capped at `max`.
fn json_limit(params: &Value, default: u64, max: u64) -> Result<u64, ToolError> {
    let Some(v) = params.get("limit") else {
        return Ok(default);
    };
    if v.is_null() {
        return Ok(default);
    }
    let n = if let Some(u) = v.as_u64() {
        u
    } else if let Some(i) = v.as_i64() {
        if i < 0 {
            return Err(tool_error(
                WebErrorCode::NetworkError,
                "`limit` must be non-negative",
            ));
        }
        i as u64
    } else if let Some(f) = v.as_f64() {
        if f < 0.0 || f.fract() != 0.0 {
            return Err(tool_error(
                WebErrorCode::NetworkError,
                "`limit` must be a non-negative whole number",
            ));
        }
        f as u64
    } else {
        return Err(tool_error(
            WebErrorCode::NetworkError,
            "`limit` must be a number",
        ));
    };
    Ok(n.min(max))
}

pub(crate) async fn op_web_search(ctx: &WebContext, params: &Value) -> ToolResult {
    let query = json_str(params, "query")?.trim();
    if query.is_empty() {
        return Err(tool_error(
            WebErrorCode::NetworkError,
            "`query` must not be empty",
        ));
    }

    let limit = json_limit(params, 5, 20)? as usize;

    let results = ctx
        .search_backend()
        .search(&ctx.client, query, limit)
        .await?;

    let arr: Vec<Value> = results
        .into_iter()
        .map(|r| {
            json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.snippet,
            })
        })
        .collect();

    Ok(ok_data(json!({ "results": arr })))
}

pub(crate) async fn op_web_fetch(ctx: &WebContext, params: &Value) -> ToolResult {
    let url_str = json_str(params, "url")?.trim();
    if url_str.is_empty() {
        return Err(tool_error(
            WebErrorCode::NetworkError,
            "`url` must not be empty",
        ));
    }

    let parsed = Url::parse(url_str)
        .map_err(|e| tool_error(WebErrorCode::NetworkError, format!("invalid URL: {e}")))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(tool_error(
            WebErrorCode::NetworkError,
            "only http and https URLs are allowed",
        ));
    }

    let out = ctx.fetch_backend().fetch(&ctx.client, &parsed).await?;

    Ok(ok_data(json!({
        "content": out.content,
        "title": out.title,
        "url": out.url,
    })))
}
