//! MCP Showcase module for browsing, installing, and managing MCP servers
//! Provides both TUI (ratatui) and CLI interfaces;

pub mod tui;
pub use tui::McpShowcaseUI;
pub use tui::McpShowcaseAction;

use crate::config::Config;
use crate::mcp::McpManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::process::{Command, Stdio};
use tokio::process::Command as TokioCommand;

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
        
        // Add community servers from mcpdirectory.app (placeholder for future API integration)
        let community_servers = [
            ("playwright", "Browser automation & E2E testing"),
            ("supabase", "RLS-aware database access"),
            ("sentry", "Error monitoring"),
            ("linear", "Issue tracking"),
            ("e2b", "Secure cloud sandbox for code execution"),
            ("mcpdirectory", "Central registry of MCP servers"),
        ];
        
        for (name, description) in community_servers {
            servers.push(McpServerInfo {
                name: name.to_string(),
                description: description.to_string(),
                installed: false,
                enabled: false,
                command: String::new(),
            });
        }
        
        servers
    }

    /// Get details for a specific server
    pub async fn get_server_details(&self, server_name: &str) -> Option<McpServerInfo> {
        let servers = self.list_servers().await;
        servers.into_iter().find(|s| s.name == server_name)
    }

    /// Install a new MCP server by name
    pub async fn install_server(&mut self, server_name: &str) -> Result<(), String> {
        // Resolve installation command from config
        let mcp_config = self.config.mcp.get_mut(server_name).ok_or_else(|| {
            format!("Server '{}' not found in config", server_name)
        })?;
        
        // If already enabled, nothing to do
        if mcp_config.enabled {
            return Ok(());
        }
        
        // Parse command into binary and args
        let (cmd, args) = resolve_spawn_command(&mcp_config.command.join(" "))?;
        
        // Spawn the installation process
        let mut child = TokioCommand::new(&cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let status = child.wait().await?;
        if !status.success() {
            return Err(format!("Installation failed for '{}' with exit code {:?}", server_name, status.code()));
        }
        
        // Mark as enabled and persist config
        mcp_config.enabled = true;
        self.config.save();
        Ok(())
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
        "playwright" => "Browser automation & E2E testing".to_string(),
        "supabase" => "RLS-aware database access".to_string(),
        "sentry" => "Error monitoring".to_string(),
        "linear" => "Issue tracking".to_string(),
        "e2b" => "Secure cloud sandbox for code execution".to_string(),
        "mcpdirectory" => "Central registry of MCP servers".to_string(),
        _ => "Unknown server".to_string(),
    }
}

/// Parse command string into binary and arguments
/// Handles formats like "npx some-tool@latest" or "pip install some-pkg"
fn resolve_spawn_command(command: &str) -> Result<(String, Vec<String>), String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".into());
    }
    let mut bin = parts[0].to_string();
    let mut args = Vec::new();
    match parts[0] {
        "npx" | "npm" | "yarn" | "pip" | "pip3" | "cargo" => {
            if parts.len() < 2 {
                return Err(format!("Missing package name for {}", parts[0]));
            }
            args = parts[1..].iter().map(|s| s.to_string()).collect();
        }
        _ => {
            args = parts[1..].iter().map(|s| s.to_string()).collect();
        }
    }
    if bin.is_empty() {
        return Err("Invalid command format".into());
    }
    Ok((bin, args))
}
