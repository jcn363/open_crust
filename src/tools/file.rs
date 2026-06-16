//! Shell and file operation tool handlers (bash, read, write, format_file)

use serde_json::Value;
use std::fs;

use crate::security;

/// Execute a shell/file tool by name. Returns Some(result) if handled.
pub fn execute_file_tool(name: &str, args: &Value) -> Option<String> {
    match name {
        "bash" => {
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            Some(match security::execute_command_safely(command) {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr)
                }
                Err(e) => format!("Security error: {}", e),
            })
        }
        "read" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            Some(match fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => format!("Error reading file: {}", e),
            })
        }
        "write" => {
            let path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            Some(match fs::write(path_str, content) {
                Ok(_) => {
                    // Auto-format on write (silently ignore formatter errors)
                    let _ = crate::formatters::format_file(std::path::Path::new(path_str));
                    format!("Successfully wrote to {}", path_str)
                }
                Err(e) => format!("Error writing file: {}", e),
            })
        }
        "format_file" => {
            let path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            Some(
                match crate::formatters::format_file(std::path::Path::new(path_str)) {
                    Ok(_) => format!("Formatted {}", path_str),
                    Err(e) => e,
                },
            )
        }
        _ => None,
    }
}
