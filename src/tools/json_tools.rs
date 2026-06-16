//! JSON tool handlers (json_validate, json_format, json_path, json_merge, etc.)

use serde_json::Value;

use crate::json_utils;

/// Execute a JSON tool by name. Returns Some(result) if handled.
pub fn execute_json_tool(name: &str, args: &Value) -> Option<String> {
    match name {
        "json_validate" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::validate_json(json_str) {
                Ok(_) => "Valid JSON".to_string(),
                Err(e) => e,
            })
        }
        "json_format" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::format_json(json_str) {
                Ok(formatted) => formatted,
                Err(e) => e,
            })
        }
        "json_path" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::get_json_path(json_str, path) {
                Ok(value) => value,
                Err(e) => e,
            })
        }
        "json_merge" => {
            let base = args.get("base").and_then(|v| v.as_str()).unwrap_or("");
            let patch = args.get("patch").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::merge_json(base, patch) {
                Ok(merged) => merged,
                Err(e) => e,
            })
        }
        "json_compact" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::compact_json(json_str) {
                Ok(compacted) => compacted,
                Err(e) => e,
            })
        }
        "json_compare" => {
            let left = args.get("left").and_then(|v| v.as_str()).unwrap_or("");
            let right = args.get("right").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::compare_json(left, right) {
                Ok(result) => result,
                Err(e) => e,
            })
        }
        "json_keys" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::get_keys(json_str) {
                Ok(keys) => keys.join(", "),
                Err(e) => e,
            })
        }
        "json_to_csv" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::to_csv(json_str) {
                Ok(csv) => csv,
                Err(e) => e,
            })
        }
        "json_array_len" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::get_array_length(json_str) {
                Ok(len) => format!("{}", len),
                Err(e) => e,
            })
        }
        "json_set_path" => {
            let json_str = args.get("json").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let new_value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
            Some(match json_utils::set_json_path(json_str, path, new_value) {
                Ok(result) => result,
                Err(e) => e,
            })
        }
        _ => None,
    }
}
