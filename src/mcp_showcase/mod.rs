//! MCP Showcase module for browsing, installing, and managing MCP servers
//! Provides both TUI (ratatui) and CLI interfaces;

pub mod tui;
pub use tui::McpShowcaseAction;
pub use tui::McpShowcaseUI;

use crate::config::Config;
use crate::mcp::McpManager;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
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
        // Find server info from the full list (includes community servers)
        let server_info = self
            .list_servers()
            .await
            .into_iter()
            .find(|s| s.name == server_name)
            .ok_or_else(|| format!("Server '{}' not found", server_name))?;

        // Determine install command - first check if it's a known community server
        let cmd_str = self.find_install_command(server_name, &server_info);

        if cmd_str.is_empty() {
            return Err(format!(
                "Server '{}' has no install command configured",
                server_name
            ));
        }

        let (cmd, args) = resolve_spawn_command(&cmd_str)?;

        // Spawn the installation process
        let mut child = TokioCommand::new(&cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn: {}", e))?;

        let status = child
            .wait()
            .await
            .map_err(|e| format!("Failed to wait for child process: {}", e))?;
        if !status.success() {
            return Err(format!(
                "Installation failed for '{}' with exit code {:?}",
                server_name,
                status.code()
            ));
        }

        // Mark as installed and enabled, persist config
        if let Some(cfg) = self.config.mcp.get_mut(server_name) {
            cfg.enabled = true;
        }
        self.config.save();
        Ok(())
    }

    /// Find the appropriate install command for a server
    fn find_install_command(&self, name: &str, info: &McpServerInfo) -> String {
        // Check if it's in config (non-community server)
        if let Some(cfg) = self.config.mcp.get(name) {
            return cfg.command.join(" ");
        }

        // Map known community servers to their install commands
        match name {
            "playwright" => "npx -y @anthropic-ai/mcp-server-playwright".to_string(),
            "supabase" => "npx -y @anthropic-ai/mcp-server-supabase".to_string(),
            "sentry" => "npx -y @anthropic-ai/mcp-server-sentry".to_string(),
            "linear" => "npx -y @anthropic-ai/mcp-server-linear".to_string(),
            "e2b" => "npx -y @anthropic-ai/mcp-server-e2b".to_string(),
            "mcpdirectory" => "npx -y @anthropic-ai/mcp-server-mcp-directory".to_string(),
            _ => {
                if info.installed && !info.description.is_empty() {
                    format!("npx -y {}", name)
                } else {
                    String::new()
                }
            }
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
}

/// Parse command string into binary and arguments
/// Handles formats like "npx some-tool@latest" or "pip install some-pkg"
#[allow(dead_code)]
fn resolve_spawn_command(command: &str) -> Result<(String, Vec<String>), String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".into());
    }
    let bin = parts[0].to_string();
    let args: Vec<String> = match parts[0] {
        "npx" | "npm" | "yarn" | "pip" | "pip3" | "cargo" => {
            if parts.len() < 2 {
                return Err(format!("Missing package name for {}", parts[0]));
            }
            parts[1..].iter().map(|s| s.to_string()).collect()
        }
        _ => parts[1..].iter().map(|s| s.to_string()).collect(),
    };
    if bin.is_empty() {
        return Err("Invalid command format".into());
    }
    Ok((bin, args))
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
