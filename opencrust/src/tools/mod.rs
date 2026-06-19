//! Tool definitions and execution dispatch
//!
//! Defines the built-in tool schema and executes tools by name (bash, file read/write,
//! glob, grep, web search, notifications, etc.). Each tool is a function that receives
//! JSON arguments and returns a string result. Integrates with MCP and LSP for
//! extended tool sets.

mod file;
mod json_tools;
mod markdown_tools;
mod notify;
mod schema;

#[cfg(test)]
mod tests;

use serde_json::Value;

pub use schema::get_tools_schema;

/// Execute a tool by name. Dispatches to the appropriate submodule handler.
pub fn execute_tool(name: &str, arguments: &Value) -> String {
    // Try each handler category in order
    if let Some(result) = file::execute_file_tool(name, arguments) {
        return result;
    }
    if let Some(result) = notify::execute_notify_tool(name, arguments) {
        return result;
    }
    if let Some(result) = json_tools::execute_json_tool(name, arguments) {
        return result;
    }
    if let Some(result) = markdown_tools::execute_markdown_tool(name, arguments) {
        return result;
    }

    format!("Unknown tool: {}", name)
}
