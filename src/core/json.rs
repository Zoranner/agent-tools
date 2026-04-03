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
