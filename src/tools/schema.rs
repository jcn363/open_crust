//! Tool schema definitions for all available tools

use serde_json::Value;

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
