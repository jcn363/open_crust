//! JSON utilities for parsing, validation, and manipulation

use serde_json::Value;

/// Validate JSON string
pub fn validate_json(json_str: &str) -> Result<Value, String> {
    serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))
}

/// Pretty print JSON
pub fn format_json(json_str: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    serde_json::to_string_pretty(&value).map_err(|e| format!("Failed to format: {}", e))
}

/// Compact JSON (no whitespace)
pub fn compact_json(json_str: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    serde_json::to_string(&value).map_err(|e| format!("Failed to compact: {}", e))
}

/// Get value from JSON path (e.g., "data.users.0.name")
pub fn get_json_path(json_str: &str, path: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = &value;
    
    for part in parts {
        current = match current.get(part) {
            Some(v) => v,
            None => return Err(format!("Path not found: {}", path)),
        };
    }
    
    serde_json::to_string(current).map_err(|e| format!("Failed to serialize: {}", e))
}

/// Set value in JSON path
pub fn set_json_path(json_str: &str, path: &str, new_value: &str) -> Result<String, String> {
    let mut value: Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    let new_val: Value = serde_json::from_str(new_value).map_err(|e| format!("Invalid new value: {}", e))?;
    
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
        
        for i in 0..last_idx {
            let key = parts[i];
            current = current.get_mut(key).ok_or_else(|| format!("Path not found: {}", key))?;
        }
        
        if let Some(obj) = current.as_object_mut() {
            obj.insert(parts[last_idx].to_string(), new_val);
        } else {
            return Err(format!("Cannot set at path: {}", path));
        }
    }
    
    serde_json::to_string_pretty(&value).map_err(|e| format!("Failed to serialize: {}", e))
}

/// Compare two JSON strings
pub fn compare_json(left_str: &str, right_str: &str) -> Result<String, String> {
    let left: Value = serde_json::from_str(left_str).map_err(|e| format!("Invalid left JSON: {}", e))?;
    let right: Value = serde_json::from_str(right_str).map_err(|e| format!("Invalid right JSON: {}", e))?;
    
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
    let mut base: Value = serde_json::from_str(base_str).map_err(|e| format!("Invalid base JSON: {}", e))?;
    let patch: Value = serde_json::from_str(patch_str).map_err(|e| format!("Invalid patch JSON: {}", e))?;
    
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
    let value: Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    
    match value.as_object() {
        Some(obj) => Ok(obj.keys().cloned().collect()),
        None => Err("JSON must be an object".to_string()),
    }
}

/// Extract array elements
pub fn get_array_length(json_str: &str) -> Result<usize, String> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    
    match value.as_array() {
        Some(arr) => Ok(arr.len()),
        None => Err("JSON must be an array".to_string()),
    }
}

/// Convert JSON to CSV (for flat arrays of objects)
pub fn to_csv(json_str: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    
    let arr = value.as_array().ok_or("JSON must be an array")?;
    
    if arr.is_empty() {
        return Ok(String::new());
    }
    
    // Collect all unique keys
    let mut keys = std::collections::HashSet::new();
    for item in arr {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                keys.insert(key.clone());
            }
        }
    }
    
    let headers: Vec<_> = keys.into_iter().collect();
    
    let mut csv = headers.join(",");
    csv.push('\n');
    
    for item in arr {
        if let Some(obj) = item.as_object() {
            for header in &headers {
                if let Some(val) = obj.get(header) {
                    csv.push_str(&val.to_string());
                }
                csv.push(',');
            }
            csv.push('\n');
        }
    }
    
    Ok(csv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        let json = r#"{"name": "test", "value": 42}"#;
        assert!(validate_json(json).is_ok());
        
        let invalid = r#"{"name": missing}"#;
        assert!(validate_json(invalid).is_err());
    }

    #[test]
    fn test_format() {
        let json = r#"{"name":"test","value":42}"#;
        let formatted = format_json(json).unwrap();
        assert!(formatted.contains('\n'));
    }

    #[test]
    fn test_get_path() {
        let json = r#"{"data": {"users": [{"name": "alice"}]}}"#;
        let result = get_json_path(json, "data.users.0.name").unwrap();
        assert!(result.contains("alice"));
    }
}