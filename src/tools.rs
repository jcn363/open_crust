//! Tool definitions and execution dispatch
//!
//! Defines the built-in tool schema and executes tools by name (bash, file read/write,
//! glob, grep, web search, notifications, etc.). Each tool is a function that receives
//! JSON arguments and returns a string result. Integrates with MCP and LSP for
//! extended tool sets.

use serde_json::Value;
use std::fs;

use crate::desktop::notifications;
use crate::json_utils;
use crate::markdown;
use crate::security;

pub fn execute_tool(name: &str, arguments: &Value) -> String {
    match name {
        "bash" => {
            let command = arguments
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match security::execute_command_safely(command) {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr)
                }
                Err(e) => format!("Security error: {}", e),
            }
        }
        "read" => {
            let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
            match fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => format!("Error reading file: {}", e),
            }
        }
        "write" => {
            let path_str = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = arguments
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match fs::write(path_str, content) {
                Ok(_) => {
                    // Auto-format on write (silently ignore formatter errors)
                    let _ = crate::formatters::format_file(std::path::Path::new(path_str));
                    format!("Successfully wrote to {}", path_str)
                }
                Err(e) => format!("Error writing file: {}", e),
            }
        }
        "format_file" => {
            let path_str = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
            match crate::formatters::format_file(std::path::Path::new(path_str)) {
                Ok(_) => format!("Formatted {}", path_str),
                Err(e) => e,
            }
        }
        "notify" => {
            let title = arguments
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Notification");
            let body = arguments.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let urgency = arguments
                .get("urgency")
                .and_then(|v| v.as_str())
                .unwrap_or("normal");

            let notif = crate::desktop::notifications::Notification::new(title, body).with_urgency(
                crate::desktop::notifications::NotificationUrgency::from_str(urgency),
            );

            // Use smart backend selection: DBus (rich features) > notify-send (fallback)
            match notifications::send_notification_smart(&notif) {
                Ok(_) => format!("Notification sent: {} - {}", title, body),
                Err(e) => format!("Failed to send notification: {}", e),
            }
        }
        "json_validate" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            match json_utils::validate_json(json_str) {
                Ok(_) => "Valid JSON".to_string(),
                Err(e) => e,
            }
        }
        "json_format" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            match json_utils::format_json(json_str) {
                Ok(formatted) => formatted,
                Err(e) => e,
            }
        }
        "json_path" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
            match json_utils::get_json_path(json_str, path) {
                Ok(value) => value,
                Err(e) => e,
            }
        }
        "json_merge" => {
            let base = arguments.get("base").and_then(|v| v.as_str()).unwrap_or("");
            let patch = arguments
                .get("patch")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match json_utils::merge_json(base, patch) {
                Ok(merged) => merged,
                Err(e) => e,
            }
        }
        "md_title" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match markdown::extract_title(md) {
                Some(title) => title,
                None => "No title found".to_string(),
            }
        }
        "md_headings" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let headings = markdown::extract_headings(md);
            if headings.is_empty() {
                "No headings found".to_string()
            } else {
                headings
                    .iter()
                    .map(|(l, t)| format!("{} {}", "#".repeat(*l as usize), t))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        "md_links" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let links = markdown::extract_links(md);
            if links.is_empty() {
                "No links found".to_string()
            } else {
                links
                    .iter()
                    .map(|(text, url)| format!("[{}]({})", text, url))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        "md_word_count" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let count = markdown::count_words(md);
            format!("{} words", count)
        }
        "json_compact" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            match json_utils::compact_json(json_str) {
                Ok(compacted) => compacted,
                Err(e) => e,
            }
        }
        "json_compare" => {
            let left = arguments.get("left").and_then(|v| v.as_str()).unwrap_or("");
            let right = arguments
                .get("right")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match json_utils::compare_json(left, right) {
                Ok(result) => result,
                Err(e) => e,
            }
        }
        "md_is_valid" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match markdown::is_valid(md) {
                true => "Valid markdown".to_string(),
                false => "Invalid markdown".to_string(),
            }
        }
        "md_to_html" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            markdown::to_html(md)
        }
        "md_extract_code" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let code_blocks = markdown::extract_code_blocks(md);
            if code_blocks.is_empty() {
                "No code blocks found".to_string()
            } else {
                code_blocks
                    .iter()
                    .map(|(_, code)| code.as_str())
                    .collect::<Vec<_>>()
                    .join("\n---\n")
            }
        }
        "md_frontmatter" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match markdown::extract_frontmatter(md) {
                Some((fm, _)) => fm,
                None => "No frontmatter found".to_string(),
            }
        }
        "md_tables" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tables = markdown::extract_tables(md);
            if tables.is_empty() {
                "No tables found".to_string()
            } else {
                let output: Vec<String> = tables
                    .iter()
                    .map(|t| {
                        t.iter()
                            .map(|row| row.join(" | "))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .collect();
                output.join("\n\n")
            }
        }
        "md_images" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let images = markdown::extract_images(md);
            if images.is_empty() {
                "No images found".to_string()
            } else {
                images
                    .iter()
                    .map(|(alt, url)| format!("![{}]({})", alt, url))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        "md_tasks" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tasks = markdown::extract_tasks(md);
            if tasks.is_empty() {
                "No tasks found".to_string()
            } else {
                tasks
                    .iter()
                    .map(|(done, text)| format!("[{}] {}", if *done { "x" } else { " " }, text))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        "md_list_items" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let items = markdown::extract_list_items(md);
            if items.is_empty() {
                "No list items found".to_string()
            } else {
                items.join("\n")
            }
        }
        "md_numbered" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let items = markdown::extract_numbered_items(md);
            if items.is_empty() {
                "No numbered items found".to_string()
            } else {
                items.join("\n")
            }
        }
        "md_quotes" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let quotes = markdown::extract_quotes(md);
            if quotes.is_empty() {
                "No blockquotes found".to_string()
            } else {
                quotes.join("\n")
            }
        }
        "md_urls" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let urls = markdown::extract_urls(md);
            if urls.is_empty() {
                "No URLs found".to_string()
            } else {
                urls.join("\n")
            }
        }
        "md_inline_code" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let code = markdown::extract_inline_code(md);
            if code.is_empty() {
                "No inline code found".to_string()
            } else {
                code.join("\n")
            }
        }
        "md_bold" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let bold = markdown::extract_bold(md);
            if bold.is_empty() {
                "No bold text found".to_string()
            } else {
                bold.join("\n")
            }
        }
        "md_italic" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let italic = markdown::extract_italic(md);
            if italic.is_empty() {
                "No italic text found".to_string()
            } else {
                italic.join("\n")
            }
        }
        "md_summary" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            markdown::get_summary(md)
        }
        "json_keys" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            match json_utils::get_keys(json_str) {
                Ok(keys) => keys.join(", "),
                Err(e) => e,
            }
        }
        "json_to_csv" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            match json_utils::to_csv(json_str) {
                Ok(csv) => csv,
                Err(e) => e,
            }
        }
        "json_array_len" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            match json_utils::get_array_length(json_str) {
                Ok(len) => format!("{}", len),
                Err(e) => e,
            }
        }
        "md_headings_tree" => {
            let md = arguments
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tree = markdown::get_headings_tree(md);
            if tree.is_empty() {
                "No headings found".to_string()
            } else {
                tree.iter()
                    .map(|(level, text, depth)| {
                        format!(
                            "{} {} {}",
                            "  ".repeat(*depth),
                            "#".repeat(*level as usize),
                            text
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        "json_set_path" => {
            let json_str = arguments.get("json").and_then(|v| v.as_str()).unwrap_or("");
            let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let new_value = arguments
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match json_utils::set_json_path(json_str, path, new_value) {
                Ok(result) => result,
                Err(e) => e,
            }
        }
        _ => format!("Unknown tool: {}", name),
    }
}

/// Get the JSON schema for all available tools.
pub fn get_tools_schema() -> Value {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "bash",
                "description": "Execute a bash command on the local system.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to run"
                        }
                    },
                    "required": ["command"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read the contents of a local file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The absolute or relative path to the file"
                        }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "write",
                "description": "Write text content to a local file, creating or overwriting it.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write"
                        }
                    },
                    "required": ["path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "format_file",
                "description": "Format a file using the appropriate formatter for its language (rustfmt for .rs, prettier for .js/.ts/.json, black for .py, etc.).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to format"
                        }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "lsp_hover",
                "description": "Get hover information for a symbol using LSP.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path" },
                        "line": { "type": "integer", "description": "Line number (0-indexed)" },
                        "character": { "type": "integer", "description": "Character position (0-indexed)" }
                    },
                    "required": ["path", "line", "character"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "lsp_find_references",
                "description": "Find all references to a symbol using LSP.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path" },
                        "line": { "type": "integer", "description": "Line number" },
                        "character": { "type": "integer", "description": "Character position" }
                    },
                    "required": ["path", "line", "character"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "notify",
                "description": "Send a desktop notification to the user. Works on Linux with desktop environments like Cinnamon, Gnome, or KDE.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Notification title"
                        },
                        "body": {
                            "type": "string",
                            "description": "Notification body message"
                        },
                        "urgency": {
                            "type": "string",
                            "description": "Urgency level: low, normal, or critical",
                            "enum": ["low", "normal", "critical"]
                        }
                    },
                    "required": ["title", "body"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "lsp_goto_definition",
                "description": "Jump to the definition of a symbol using LSP.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path" },
                        "line": { "type": "integer", "description": "Line number (0-indexed)" },
                        "character": { "type": "integer", "description": "Character position (0-indexed)" }
                    },
                    "required": ["path", "line", "character"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for information.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The search query" }
                    },
                    "required": ["query"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "fetch_url",
                "description": "Fetch content from a URL and convert it to markdown.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "The URL to fetch" }
                    },
                    "required": ["url"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "pin",
                "description": "Pin a file to the context so it stays in the prompt.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The path to the file to pin" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "unpin",
                "description": "Unpin a file from the context.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The path to the file to unpin" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "create_plan",
                "description": "Create a multi-step plan for a complex task.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string", "description": "The title of the plan" },
                        "steps": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "A list of steps to complete the task"
                        }
                    },
                    "required": ["title", "steps"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "mark_step_complete",
                "description": "Mark a step of the current plan as complete.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "index": { "type": "integer", "description": "The 0-indexed position of the step to complete" }
                    },
                    "required": ["index"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "semantic_search",
                "description": "Perform a semantic search across the codebase to find relevant snippets using vector embeddings.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The search query or concept to find" },
                        "top_k": { "type": "integer", "description": "Number of top results to return (default: 5)" }
                    },
                    "required": ["query"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "index_codebase",
                "description": "Index the codebase for semantic search using vector embeddings. Must be called before semantic_search will return results.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "root": { "type": "string", "description": "Root directory to index (default: current directory)" }
                    },
                    "required": []
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "global_search_replace",
                "description": "Perform a global search and replace across the codebase.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "The regex pattern to search for" },
                        "replacement": { "type": "string", "description": "The replacement string" },
                        "include": { "type": "string", "description": "Glob pattern for files to include (e.g. '*.rs')" }
                    },
                    "required": ["pattern", "replacement"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "skill",
                "description": "Load the full content of a reusable skill.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "The name of the skill to load" }
                    },
                    "required": ["name"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "lsp_completion",
                "description": "Get code completion suggestions at a specific position in a file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The file path" },
                        "line": { "type": "integer", "description": "Line number (0-based)" },
                        "character": { "type": "integer", "description": "Character offset (0-based)" }
                    },
                    "required": ["path", "line", "character"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "lsp_diagnostics",
                "description": "Get diagnostics (errors, warnings) for a file from the LSP server.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The file path" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "lsp_formatting",
                "description": "Format a file using the LSP server's formatting provider.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The file path" }
                    },
                    "required": ["path"]
                }
            }
        }
    ])
}

#[cfg(test)]
mod tests {
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
}
