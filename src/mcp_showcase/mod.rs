//! MCP Showcase module for browsing, installing, and managing MCP servers
//! Provides both TUI (ratatui) and CLI interfaces;

pub mod tui;
pub use tui::McpShowcaseUI;
pub use tui::McpShowcaseAction;


use crate::config::Config;
use crate::mcp::McpManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP Showcase main struct
#[allow(dead_code)]
pub struct McpShowcase {
    config: Config,
    mcp_manager: Arc<Mutex<McpManager>>,
}

#[allow(dead_code)]
impl McpShowcase {
    /// Create a new MCP Showcase instance
    pub fn new(config: Config, mcp_manager: Arc<Mutex<McpManager>>) -> Self {
        Self {
            config,
            mcp_manager,
        }
    }

    /// List all available MCP servers (recommended + installed)
    pub async fn list_servers(&self) -> Vec<McpServerInfo> {
        let mut servers = Vec::new();
        
        // Add recommended servers from config
        for (name, mcp_config) in &self.config.mcp {
            servers.push(McpServerInfo {
                name: name.clone(),
                description: get_server_description(name),
                installed: true,
                enabled: mcp_config.enabled,
                command: mcp_config.command.join(" "),
            });
        }
        
        // TODO: Add more community servers from mcpdirectory.app
        
        servers
    }

    /// Get details for a specific server
    pub async fn get_server_details(&self, server_name: &str) -> Option<McpServerInfo> {
        let servers = self.list_servers().await;
        servers.into_iter().find(|s| s.name == server_name)
    }

    /// Install a new MCP server by name
    pub async fn install_server(&mut self, server_name: &str) -> Result<(), String> {
        // TODO: Implement installation logic (npx, pip, etc.)
        // For now, just enable if already in config
        if let Some(mcp_config) = self.config.mcp.get_mut(server_name) {
            mcp_config.enabled = true;
            self.config.save();
            Ok(())
        } else {
            Err(format!("Server '{}' not found in config", server_name))
        }
    }
}

/// Public information about an MCP server
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub enabled: bool,
    #[allow(dead_code)]
    pub command: String, // Stored for future install/uninstall operations
}

/// Information about an MCP tool
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP tool execution result
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub execution_time_ms: u64,
}

/// Parse MCP tool list response into ToolInfo structs
#[allow(dead_code)]
pub fn parse_tools_from_response(tools: &[serde_json::Value]) -> Vec<ToolInfo> {
    tools.iter().map(|tool| {
        ToolInfo {
            name: tool.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            description: tool.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            input_schema: tool.get("inputSchema").cloned().unwrap_or(serde_json::json!({})),
        }
    }).collect()
}

/// Get a human-readable description for known MCP servers
pub fn get_server_description(server_name: &str) -> String {
    match server_name {
        "context7" => "Version-accurate library docs (eliminates API hallucinations)".to_string(),
        "github" => "Repository management, issues, PRs, CI/CD".to_string(),
        "brave-search" => "Privacy-focused web search".to_string(),
        "postgres" => "Natural language DB queries".to_string(),
        "filesystem" => "Enhanced file operations".to_string(),
        _ => "MCP Server".to_string(),
    }
}
