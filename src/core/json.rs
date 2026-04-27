//! 工具成功响应外壳与通用 JSON 参数解析。

use serde_json::{json, Value};

use crate::tool::ToolError;

/// `{ "success": true, "data": ... }`，与各工具 `execute` 成功返回值一致。
pub(crate) fn ok_data(data: Value) -> Value {
    json!({
        "success": true,
        "data": data,
    })
}

/// 读取必填字符串参数；缺失或非 `string` 时返回 `INVALID_PATH`（与 `fs` / `md` / `find` 等模块约定一致）。
pub(crate) fn json_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError {
            code: "INVALID_PATH".into(),
            message: format!("missing or invalid `{key}`"),
        })
}

fn invalid_param(key: &str, expected: &str) -> ToolError {
    ToolError {
        code: "INVALID_PATH".into(),
        message: format!("invalid `{key}`: expected {expected}"),
    }
}

/// 读取可选字符串参数；缺失时返回 `Ok(None)`，存在但非 `string` 时返回 `INVALID_PATH`。
pub(crate) fn json_str_opt<'a>(params: &'a Value, key: &str) -> Result<Option<&'a str>, ToolError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_str()
            .map(Some)
            .ok_or_else(|| invalid_param(key, "string")),
    }
}

/// 读取可选布尔参数；缺失时返回 `Ok(None)`，存在但非 `boolean` 时返回 `INVALID_PATH`。
pub(crate) fn json_bool_opt(params: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_bool()
            .map(Some)
            .ok_or_else(|| invalid_param(key, "boolean")),
    }
}

/// 读取可选无符号整数参数；缺失时返回 `Ok(None)`，存在但非 JSON `integer` 时返回 `INVALID_PATH`。
pub(crate) fn json_u64_opt(params: &Value, key: &str) -> Result<Option<u64>, ToolError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_u64()
            .map(Some)
            .ok_or_else(|| invalid_param(key, "integer")),
    }
}

/// 读取可选字符串数组；缺失时返回 `Ok(None)`，存在但非 `string[]` 时返回 `INVALID_PATH`。
pub(crate) fn json_string_array_opt(
    params: &Value,
    key: &str,
) -> Result<Option<Vec<String>>, ToolError> {
    let Some(v) = params.get(key) else {
        return Ok(None);
    };
    let arr = v
        .as_array()
        .ok_or_else(|| invalid_param(key, "array of strings"))?;

    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| invalid_param(key, "array of strings"))?;
        out.push(s.to_string());
    }
    Ok(Some(out))
}
