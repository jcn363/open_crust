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
    let result = compact_json(
        r#"{
            "a": 1,
            "b": 2
        }"#,
    )
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
