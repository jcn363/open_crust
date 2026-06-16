//! Tests for tools module

use super::*;

// --- get_tools_schema ---

#[test]
fn schema_returns_valid_json() {
    let schema = get_tools_schema();
    assert!(schema.is_object() || schema.is_array());
}

#[test]
fn schema_contains_expected_tools() {
    let schema = get_tools_schema();
    let names = schema
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry["function"]["name"].as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"bash"));
    assert!(names.contains(&"read"));
    assert!(names.contains(&"write"));
    assert!(names.contains(&"notify"));
    assert!(names.contains(&"web_search"));
    assert!(names.contains(&"semantic_search"));
    assert!(names.contains(&"create_plan"));
}

#[test]
fn schema_tool_has_required_fields() {
    let schema = get_tools_schema();
    for entry in schema.as_array().unwrap() {
        let func = &entry["function"];
        assert!(func["name"].is_string(), "Tool missing name");
        assert!(
            func["description"].is_string(),
            "Tool {} missing description",
            func["name"]
        );
        assert!(
            func["parameters"].is_object(),
            "Tool {} missing parameters",
            func["name"]
        );
    }
}

// --- execute_tool: pure wrappers ---

#[test]
fn execute_unknown_tool_returns_error_message() {
    let result = execute_tool("nonexistent_tool", &serde_json::json!({}));
    assert_eq!(result, "Unknown tool: nonexistent_tool");
}

#[test]
fn execute_json_validate_valid_json() {
    let result = execute_tool("json_validate", &serde_json::json!({"json": "{\"a\":1}"}));
    assert_eq!(result, "Valid JSON");
}

#[test]
fn execute_json_validate_invalid_json() {
    let result = execute_tool("json_validate", &serde_json::json!({"json": "{bad}"}));
    assert_eq!(
        result,
        "Invalid JSON: key must be a string at line 1 column 2"
    );
}

#[test]
fn execute_json_format_pretty_print() {
    let result = execute_tool("json_format", &serde_json::json!({"json": "{\"a\":1}"}));
    assert!(result.contains('\n'));
}

#[test]
fn execute_json_path_nested() {
    let json_str = r#"{"data":{"user":"alice"}}"#;
    let result = execute_tool(
        "json_path",
        &serde_json::json!({"json": json_str, "path": "data.user"}),
    );
    assert!(result.contains("alice"));
}

#[test]
fn execute_json_compact() {
    let result = execute_tool("json_compact", &serde_json::json!({"json": "{\"a\": 1}"}));
    assert_eq!(result, "{\"a\":1}");
}

#[test]
fn execute_json_compare_equal() {
    let result = execute_tool(
        "json_compare",
        &serde_json::json!({"left": "{\"a\":1}", "right": "{\"a\":1}"}),
    );
    assert!(result.contains("equal"));
}

#[test]
fn execute_json_keys() {
    let result = execute_tool(
        "json_keys",
        &serde_json::json!({"json": "{\"a\":1,\"b\":2}"}),
    );
    assert!(result.contains("a"));
    assert!(result.contains("b"));
}

#[test]
fn execute_json_array_len() {
    let result = execute_tool("json_array_len", &serde_json::json!({"json": "[1,2,3]"}));
    assert_eq!(result, "3");
}

#[test]
fn execute_json_set_path() {
    let result = execute_tool(
        "json_set_path",
        &serde_json::json!({"json": "{\"a\":1}", "path": "b", "value": "2"}),
    );
    assert!(result.contains("\"b\": 2"));
}

#[test]
fn execute_json_to_csv() {
    let result = execute_tool("json_to_csv", &serde_json::json!({"json": "[{\"x\":1}]"}));
    assert!(result.contains("x"));
}

#[test]
fn execute_md_title() {
    let result = execute_tool(
        "md_title",
        &serde_json::json!({"markdown": "# Hello\nworld"}),
    );
    assert_eq!(result, "Hello");
}

#[test]
fn execute_md_title_no_title() {
    let result = execute_tool("md_title", &serde_json::json!({"markdown": "plain text"}));
    assert_eq!(result, "No title found");
}

#[test]
fn execute_md_headings() {
    let result = execute_tool(
        "md_headings",
        &serde_json::json!({"markdown": "# H1\n## H2"}),
    );
    assert!(result.contains("H1"));
    assert!(result.contains("H2"));
}

#[test]
fn execute_md_word_count() {
    let result = execute_tool(
        "md_word_count",
        &serde_json::json!({"markdown": "hello world"}),
    );
    assert_eq!(result, "2 words");
}

#[test]
fn execute_md_is_valid() {
    let result = execute_tool("md_is_valid", &serde_json::json!({"markdown": "# Hello"}));
    assert_eq!(result, "Valid markdown");
}

#[test]
fn execute_tool_missing_arg_does_not_panic() {
    let result = execute_tool("json_validate", &serde_json::json!({}));
    assert!(!result.is_empty());
}

#[test]
fn execute_md_frontmatter() {
    let md = "---\ntitle: Test\n---\n\nContent";
    let result = execute_tool("md_frontmatter", &serde_json::json!({"markdown": md}));
    assert!(result.contains("title"));
}

#[test]
fn execute_md_links() {
    let md = "Text with [a link](https://example.com)";
    let result = execute_tool("md_links", &serde_json::json!({"markdown": md}));
    assert!(result.contains("example.com"));
}

#[test]
fn execute_md_code_blocks() {
    let md = "```rust\nfn main() {}\n```";
    let result = execute_tool("md_extract_code", &serde_json::json!({"markdown": md}));
    assert!(result.contains("fn main()"));
}

#[test]
fn execute_md_tables() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n\n";
    let result = execute_tool("md_tables", &serde_json::json!({"markdown": md}));
    assert!(
        !result.is_empty(),
        "result was empty (expected table output)"
    );
    // Should return as joined rows
    let expected = "1 | 2";
    assert!(
        result.contains(expected),
        "expected '{}' in result, got: {:?}",
        expected,
        result
    );
}

#[test]
fn execute_md_tasks() {
    let md = "- [x] done\n- [ ] todo";
    let result = execute_tool("md_tasks", &serde_json::json!({"markdown": md}));
    assert!(result.contains("[x]"));
    assert!(result.contains("[ ]"));
}

#[test]
fn execute_md_summary() {
    let md = "# Big Title\n\nLots of content here.";
    let result = execute_tool("md_summary", &serde_json::json!({"markdown": md}));
    assert!(!result.is_empty());
}

#[test]
fn execute_md_quotes() {
    let md = "> A wise quote";
    let result = execute_tool("md_quotes", &serde_json::json!({"markdown": md}));
    assert!(result.contains("A wise quote"));
}

// --- bash tool: security integration ---

#[test]
fn bash_safe_command_executes() {
    let result = execute_tool("bash", &serde_json::json!({"command": "echo hello"}));
    assert!(
        result.contains("hello"),
        "Expected 'hello' in output: {}",
        result
    );
}

#[test]
fn bash_dangerous_command_rejected() {
    let result = execute_tool("bash", &serde_json::json!({"command": "rm -rf /"}));
    assert!(
        result.contains("Security error") || result.contains("error"),
        "Expected security error for dangerous command: {}",
        result
    );
}

#[test]
fn bash_command_injection_rejected() {
    let result = execute_tool(
        "bash",
        &serde_json::json!({"command": "echo test; rm -rf /"}),
    );
    assert!(
        result.contains("Security error") || result.contains("error"),
        "Expected security error for command injection: {}",
        result
    );
}

#[test]
fn bash_pipe_injection_rejected() {
    let result = execute_tool("bash", &serde_json::json!({"command": "echo test | sh"}));
    assert!(
        result.contains("Security error") || result.contains("error"),
        "Expected security error for pipe injection: {}",
        result
    );
}

#[test]
fn bash_empty_command_returns_output() {
    let result = execute_tool("bash", &serde_json::json!({"command": ""}));
    // Empty command should either return an error or empty output, not panic
    assert!(!result.is_empty());
}

#[test]
fn bash_missing_command_arg_returns_output() {
    let result = execute_tool("bash", &serde_json::json!({}));
    // Missing command arg defaults to empty string
    assert!(!result.is_empty());
}

#[test]
fn bash_command_substitution_rejected() {
    let result = execute_tool("bash", &serde_json::json!({"command": "echo $(whoami)"}));
    assert!(
        result.contains("Security error") || result.contains("error"),
        "Expected security error for command substitution: {}",
        result
    );
}

#[test]
fn bash_backtick_substitution_rejected() {
    let result = execute_tool("bash", &serde_json::json!({"command": "echo `whoami`"}));
    assert!(
        result.contains("Security error") || result.contains("error"),
        "Expected security error for backtick substitution: {}",
        result
    );
}
