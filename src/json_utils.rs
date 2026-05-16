//! JSON utilities for parsing, validation, and manipulation

use serde_json::Value;

/// Validate JSON string
pub fn validate_json(json_str: &str) -> Result<Value, String> {
    serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))
}

/// Pretty print JSON
pub fn format_json(json_str: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    serde_json::to_string_pretty(&value).map_err(|e| format!("Failed to format: {}", e))
}

/// Compact JSON (no whitespace)
pub fn compact_json(json_str: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    serde_json::to_string(&value).map_err(|e| format!("Failed to compact: {}", e))
}

/// Get value from JSON path (e.g., "data.users.0.name")
pub fn get_json_path(json_str: &str, path: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = &value;

    for part in parts {
        // Try to parse as array index first, then fall back to object key
        current = if let Ok(index) = part.parse::<usize>() {
            match current.get(index) {
                Some(v) => v,
                None => return Err(format!("Path not found: {}", path)),
            }
        } else {
            match current.get(part) {
                Some(v) => v,
                None => return Err(format!("Path not found: {}", path)),
            }
        };
    }

    serde_json::to_string(current).map_err(|e| format!("Failed to serialize: {}", e))
}

/// Set value in JSON path
pub fn set_json_path(json_str: &str, path: &str, new_value: &str) -> Result<String, String> {
    let mut value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    let new_val: Value =
        serde_json::from_str(new_value).map_err(|e| format!("Invalid new value: {}", e))?;

    let parts: Vec<&str> = path.split('.').collect();

    // Navigate to parent and set
    if parts.len() == 1 {
        if let Some(obj) = value.as_object_mut() {
            obj.insert(parts[0].to_string(), new_val);
        } else {
            return Err("Root must be object".to_string());
        }
    } else {
        let last_idx = parts.len() - 1;
        let mut current = &mut value;

        for key in parts.iter().take(last_idx) {
            // Try to parse as array index, otherwise use as object key
            if let Ok(idx) = key.parse::<usize>() {
                current = current
                    .get_mut(idx)
                    .ok_or_else(|| format!("Path not found: {}", key))?;
            } else {
                current = current
                    .get_mut(key)
                    .ok_or_else(|| format!("Path not found: {}", key))?;
            }
        }

        // Set the value at the final path component
        if let Ok(idx) = parts[last_idx].parse::<usize>() {
            if let Some(arr) = current.as_array_mut() {
                if idx < arr.len() {
                    arr[idx] = new_val;
                } else {
                    return Err(format!("Array index out of bounds: {}", idx));
                }
            } else {
                return Err(format!(
                    "Cannot set array index on non-array at path: {}",
                    path
                ));
            }
        } else if let Some(obj) = current.as_object_mut() {
            obj.insert(parts[last_idx].to_string(), new_val);
        } else {
            return Err(format!("Cannot set at path: {}", path));
        }
    }

    serde_json::to_string_pretty(&value).map_err(|e| format!("Failed to serialize: {}", e))
}

/// Compare two JSON strings
pub fn compare_json(left_str: &str, right_str: &str) -> Result<String, String> {
    let left: Value =
        serde_json::from_str(left_str).map_err(|e| format!("Invalid left JSON: {}", e))?;
    let right: Value =
        serde_json::from_str(right_str).map_err(|e| format!("Invalid right JSON: {}", e))?;

    if left == right {
        Ok("JSON values are equal".to_string())
    } else {
        // Find differences
        let mut diffs = Vec::new();
        find_differences(&left, &right, "", &mut diffs);

        if diffs.is_empty() {
            Ok("JSON values are equal".to_string())
        } else {
            Ok(format!("Differences found:\n{}", diffs.join("\n")))
        }
    }
}

fn find_differences(left: &Value, right: &Value, path: &str, diffs: &mut Vec<String>) {
    match (left, right) {
        (Value::Object(l), Value::Object(r)) => {
            let all_keys: std::collections::HashSet<_> = l.keys().chain(r.keys()).collect();
            for key in all_keys {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                match (l.get(key), r.get(key)) {
                    (Some(lv), Some(rv)) => find_differences(lv, rv, &new_path, diffs),
                    (Some(_), None) => diffs.push(format!("{}: missing in right", new_path)),
                    (None, Some(_)) => diffs.push(format!("{}: missing in left", new_path)),
                    _ => {}
                }
            }
        }
        (Value::Array(l), Value::Array(r)) if l.len() == r.len() => {
            for (i, (lv, rv)) in l.iter().zip(r.iter()).enumerate() {
                let new_path = format!("{}[{}]", path, i);
                find_differences(lv, rv, &new_path, diffs);
            }
        }
        _ if left != right => {
            diffs.push(format!("{}: {} != {}", path, left, right));
        }
        _ => {}
    }
}

/// Merge two JSON objects
pub fn merge_json(base_str: &str, patch_str: &str) -> Result<String, String> {
    let mut base: Value =
        serde_json::from_str(base_str).map_err(|e| format!("Invalid base JSON: {}", e))?;
    let patch: Value =
        serde_json::from_str(patch_str).map_err(|e| format!("Invalid patch JSON: {}", e))?;

    merge_values(&mut base, &patch);

    serde_json::to_string_pretty(&base).map_err(|e| format!("Failed to serialize: {}", e))
}

fn merge_values(base: &mut Value, patch: &Value) {
    if let (Some(base_obj), Some(patch_obj)) = (base.as_object_mut(), patch.as_object()) {
        for (key, value) in patch_obj.iter() {
            match base_obj.get_mut(key) {
                Some(base_val) => merge_values(base_val, value),
                None => {
                    base_obj.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        *base = patch.clone();
    }
}

/// Extract keys from JSON object
pub fn get_keys(json_str: &str) -> Result<Vec<String>, String> {
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    match value.as_object() {
        Some(obj) => Ok(obj.keys().cloned().collect()),
        None => Err("JSON must be an object".to_string()),
    }
}

/// Extract array elements
pub fn get_array_length(json_str: &str) -> Result<usize, String> {
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    match value.as_array() {
        Some(arr) => Ok(arr.len()),
        None => Err("JSON must be an array".to_string()),
    }
}

/// Convert JSON to CSV (for flat arrays of objects)
pub fn to_csv(json_str: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    let arr = value.as_array().ok_or("JSON must be an array")?;

    if arr.is_empty() {
        return Ok(String::new());
    }

    // Collect all unique keys in insertion order
    let mut keys = std::collections::BTreeSet::new();
    for item in arr {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                keys.insert(key.clone());
            }
        }
    }

    let headers: Vec<_> = keys.into_iter().collect();
    let mut csv = String::new();

    // Header row
    for (i, header) in headers.iter().enumerate() {
        if i > 0 {
            csv.push(',');
        }
        csv.push_str(&escape_csv(header));
    }
    csv.push('\n');

    // Data rows
    for item in arr {
        if let Some(obj) = item.as_object() {
            for (i, header) in headers.iter().enumerate() {
                if i > 0 {
                    csv.push(',');
                }
                let val = match obj.get(header) {
                    Some(v) => match v {
                        Value::String(s) => escape_csv(s),
                        other => other.to_string(),
                    },
                    None => String::new(),
                };
                csv.push_str(&val);
            }
            csv.push('\n');
        }
    }

    Ok(csv)
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_json ---

    #[test]
    fn validate_should_accept_valid_json() {
        assert!(validate_json(r#"{"a":1}"#).is_ok());
    }

    #[test]
    fn validate_should_reject_invalid_json() {
        assert!(validate_json(r#"{invalid}"#).is_err());
    }

    // --- format_json ---

    #[test]
    fn format_should_pretty_print() {
        let result = format_json(r#"{"a":1,"b":2}"#).unwrap();
        assert!(result.contains('\n'));
        assert!(result.contains("  \"a\""));
    }

    #[test]
    fn format_should_error_on_invalid_input() {
        assert!(format_json("not json").is_err());
    }

    // --- compact_json ---

    #[test]
    fn compact_should_remove_whitespace() {
        let result =
            compact_json(r#"{
            "a": 1,
            "b": 2
        }"#)
            .unwrap();
        assert!(!result.contains('\n'));
        assert!(result.contains(r#"{"a":1,"b":2}"#) || result.contains(r#"{"b":2,"a":1}"#));
    }

    #[test]
    fn compact_should_error_on_invalid_input() {
        assert!(compact_json("bad").is_err());
    }

    // --- get_json_path ---

    #[test]
    fn get_path_should_navigate_nested_objects() {
        let json = r#"{"data": {"users": [{"name": "alice"}]}}"#;
        let result = get_json_path(json, "data.users.0.name").unwrap();
        assert!(result.contains("alice"));
    }

    #[test]
    fn get_path_should_return_top_level_key() {
        let json = r#"{"x": 10}"#;
        let result = get_json_path(json, "x").unwrap();
        assert!(result.contains("10"));
    }

    #[test]
    fn get_path_should_error_on_missing_path() {
        let json = r#"{"a":1}"#;
        assert!(get_json_path(json, "b").is_err());
    }

    #[test]
    fn get_path_should_error_on_invalid_json() {
        assert!(get_json_path("bad", "x").is_err());
    }

    // --- set_json_path ---

    #[test]
    fn set_path_should_replace_existing_field() {
        let json = r#"{"name": "old"}"#;
        let result = set_json_path(json, "name", r#""new""#).unwrap();
        assert!(result.contains(r#""new""#));
    }

    #[test]
    fn set_path_should_add_new_field() {
        let json = r#"{"a":1}"#;
        let result = set_json_path(json, "b", "2").unwrap();
        assert!(result.contains(r#""b": 2"#));
    }

    #[test]
    fn set_path_should_update_array_element() {
        let json = r#"{"items": [1, 2, 3]}"#;
        let result = set_json_path(json, "items.1", "99").unwrap();
        assert!(result.contains("99"));
        assert!(!result.contains(r#""items": [1, 2, 3]"#));
    }

    #[test]
    fn set_path_should_error_on_invalid_json() {
        assert!(set_json_path("bad", "x", "1").is_err());
    }

    #[test]
    fn set_path_should_error_on_array_index_out_of_bounds() {
        let json = r#"[1, 2]"#;
        // Root array
        let result = set_json_path(json, "5", "99");
        assert!(result.is_err());
    }

    // --- compare_json ---

    #[test]
    fn compare_equal_json_should_return_equal() {
        let result = compare_json(r#"{"a":1}"#, r#"{"a":1}"#).unwrap();
        assert_eq!(result, "JSON values are equal");
    }

    #[test]
    fn compare_different_json_should_show_diffs() {
        let result = compare_json(r#"{"a":1}"#, r#"{"a":2}"#).unwrap();
        assert!(result.contains("Differences found"));
    }

    #[test]
    fn compare_json_with_missing_keys_should_report() {
        let result = compare_json(r#"{"a":1,"b":2}"#, r#"{"a":1}"#).unwrap();
        assert!(result.contains("missing"));
    }

    #[test]
    fn compare_json_should_error_on_invalid_input() {
        assert!(compare_json("bad", r#"{}"#).is_err());
    }

    // --- merge_json ---

    #[test]
    fn merge_should_add_new_keys() {
        let result = merge_json(r#"{"a":1}"#, r#"{"b":2}"#).unwrap();
        assert!(result.contains(r#""a": 1"#));
        assert!(result.contains(r#""b": 2"#));
    }

    #[test]
    fn merge_should_overwrite_existing_keys() {
        let result = merge_json(r#"{"a":1}"#, r#"{"a":2}"#).unwrap();
        assert!(result.contains(r#""a": 2"#));
    }

    #[test]
    fn merge_should_error_on_invalid_base() {
        assert!(merge_json("bad", r#"{}"#).is_err());
    }

    #[test]
    fn merge_should_error_on_invalid_patch() {
        assert!(merge_json(r#"{}"#, "bad").is_err());
    }

    // --- get_keys ---

    #[test]
    fn get_keys_should_return_all_keys() {
        let result = get_keys(r#"{"a":1,"b":2}"#).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"b".to_string()));
    }

    #[test]
    fn get_keys_should_error_on_non_object() {
        assert!(get_keys(r#"[1,2,3]"#).is_err());
    }

    #[test]
    fn get_keys_should_error_on_invalid_json() {
        assert!(get_keys("bad").is_err());
    }

    // --- get_array_length ---

    #[test]
    fn get_array_length_should_return_count() {
        let result = get_array_length(r#"[10,20,30]"#).unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn get_array_length_should_return_zero_for_empty() {
        let result = get_array_length(r#"[]"#).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn get_array_length_should_error_on_non_array() {
        assert!(get_array_length(r#"{}"#).is_err());
    }

    #[test]
    fn get_array_length_should_error_on_invalid_json() {
        assert!(get_array_length("bad").is_err());
    }

    // --- to_csv ---

    #[test]
    fn to_csv_should_produce_header_and_rows() {
        let json = r#"[
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]"#;
        let result = to_csv(json).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
    }

    #[test]
    fn to_csv_should_escape_commas() {
        let json = r#"[
            {"city": "San Francisco, CA"}
        ]"#;
        let result = to_csv(json).unwrap();
        assert!(result.contains("\"San Francisco, CA\""));
    }

    #[test]
    fn to_csv_should_return_empty_for_empty_array() {
        let result = to_csv(r#"[]"#).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn to_csv_should_error_on_non_array() {
        assert!(to_csv(r#"{}"#).is_err());
    }
}
