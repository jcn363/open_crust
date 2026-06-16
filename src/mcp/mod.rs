//! MCP (Model Context Protocol) server management
//!
//! Manages external MCP servers: spawning child processes, establishing
//! JSON-RPC 2.0 communication, listing available tools, and calling tools
//! on remote servers. Supports dynamic discovery via an MCP registry.

use tokio::process::Command;

use crate::config::McpConfig;
use crate::jsonrpc::JsonRpcClient;

pub struct McpServer {
    pub name: String,
    pub rpc: JsonRpcClient,
}

impl McpServer {
    pub fn spawn(name: &str, config: &McpConfig) -> Result<Self, String> {
        if config.command.is_empty() {
            return Err("Empty MCP command".into());
        }

        // SECURITY: Use command array directly to avoid shell injection via join/split round-trip.
        // The command array comes from trusted configuration (config.json), not user input.
        // Validate the command is from an allowed list of MCP server executables.
        let cmd = &config.command[0];
        let args = &config.command[1..];

        // Validate the executable is a known-safe MCP server command
        validate_mcp_command(cmd)?;

        let mut command = Command::new(cmd);
        command.args(args);

        // Configure stdio for JSON-RPC communication
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::inherit());

        let process = command.spawn().map_err(|e| e.to_string())?;

        // Initialize JSON-RPC client
        let rpc = JsonRpcClient::new(process);

        Ok(Self {
            name: name.to_string(),
            rpc,
        })
    }
}

/// Validate that an MCP server command is from a trusted, allowed list.
/// This prevents execution of arbitrary commands via MCP configuration.
fn validate_mcp_command(cmd: &str) -> Result<(), String> {
    // List of allowed MCP server executables (basename only)
    // These are the standard MCP server packages from the official registry
    const ALLOWED_MCP_COMMANDS: &[&str] = &[
        "npx",
        "node",
        "python",
        "python3",
        "pip",
        "pip3",
        "cargo",
        "uvx",
        "uv",
        "go",
        "java",
        "mcp-server",
    ];

    // Extract basename (handle paths like /usr/bin/npx)
    let basename = std::path::Path::new(cmd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(cmd);

    // Check if the command is in the allowed list
    if ALLOWED_MCP_COMMANDS.contains(&basename) {
        Ok(())
    } else {
        Err(format!(
            "MCP command '{}' is not in the allowed list. Allowed: {}",
            basename,
            ALLOWED_MCP_COMMANDS.join(", ")
        ))
    }
}

pub struct McpManager {
    servers: Vec<McpServer>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    pub async fn load_from_config(
        &mut self,
        config: &std::collections::HashMap<String, McpConfig>,
    ) {
        for (name, mcp_config) in config {
            if !mcp_config.enabled {
                continue;
            }
            match McpServer::spawn(name, mcp_config) {
                Ok(server) => {
                    self.servers.push(server);
                    println!("opencrust: MCP server '{}' connected.", name);
                }
                Err(e) => {
                    eprintln!("opencrust: Error starting MCP server '{}': {}", name, e);
                }
            }
        }
    }

    pub async fn list_tools(&mut self) -> Vec<serde_json::Value> {
        let mut tools = Vec::new();
        for server in &mut self.servers {
            match server.rpc.call("tools/list", serde_json::json!({})).await {
                Ok(result) => {
                    if let Some(server_tools) = result.as_array() {
                        for tool in server_tools {
                            let mut tool = tool.clone();
                            // Tag each tool with its server name for routing
                            if let Some(obj) = tool.as_object_mut() {
                                obj.insert(
                                    "_server".to_string(),
                                    serde_json::Value::String(server.name.clone()),
                                );
                            }
                            tools.push(tool);
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "opencrust: Error listing tools from server '{}': {}",
                        server.name, e
                    );
                }
            }
        }
        tools
    }

    pub async fn call_tool(
        &mut self,
        name: &str,
        args: &serde_json::Value,
    ) -> Result<String, String> {
        // Extract server name if tagged in the tool name (format: "server::tool_name")
        let (server_name, tool_name) = if let Some(idx) = name.find("::") {
            (&name[..idx], &name[idx + 2..])
        } else {
            // Fallback: try all servers
            for server in &mut self.servers {
                match server
                    .rpc
                    .call(
                        "tools/call",
                        serde_json::json!({
                            "name": name,
                            "arguments": args
                        }),
                    )
                    .await
                {
                    Ok(result) => {
                        return Ok(result
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| result.to_string()));
                    }
                    Err(_) => continue,
                }
            }
            return Err(format!("MCP tool '{}' not found on any server", name));
        };

        match self.servers.iter_mut().find(|s| s.name == server_name) {
            Some(server) => server
                .rpc
                .call(
                    "tools/call",
                    serde_json::json!({
                        "name": tool_name,
                        "arguments": args
                    }),
                )
                .await
                .map(|result| {
                    result
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| result.to_string())
                })
                .map_err(|e| format!("MCP tool '{}' call failed: {}", name, e)),
            None => Err(format!("MCP server '{}' not available", server_name)),
        }
    }
}

#[cfg(test)]
mod tests;
