use serde_json::Value;
use std::fs;
use std::process::Command;

pub fn execute_tool(name: &str, arguments: &Value) -> String {
    match name {
        "bash" => {
            let command = arguments.get("command").and_then(|v| v.as_str()).unwrap_or("");
            match Command::new("sh").arg("-c").arg(command).output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr)
                }
                Err(e) => format!("Error executing command: {}", e),
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
            let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or("");
            match fs::write(path_str, content) {
                Ok(_) => {
                    crate::formatters::format_file(std::path::Path::new(path_str));
                    format!("Successfully wrote to {}", path_str)
                }
                Err(e) => format!("Error writing file: {}", e),
            }
        }
        _ => format!("Unknown tool: {}", name),
    }
}

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
                "name": "lsp_type_definition",
                "description": "Go to the type definition of a symbol using LSP.",
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
                "name": "task",
                "description": "Spawn a subagent to solve a specific sub-problem.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "The specific instruction for the subagent" }
                    },
                    "required": ["prompt"]
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
                "description": "Perform a semantic search across the codebase to find relevant snippets.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The search query or concept to find" }
                    },
                    "required": ["query"]
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
        }
    ])
}
