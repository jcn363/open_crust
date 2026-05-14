use tokio::process::Command;

use crate::config::McpConfig;
use crate::jsonrpc::JsonRpcClient;

pub struct McpServer {
    pub name: String,
    pub rpc: JsonRpcClient,
}

impl McpServer {
    pub async fn spawn(name: &str, config: &McpConfig) -> Result<Self, String> {
        if config.command.is_empty() {
            return Err("Empty MCP command".into());
        }

        // Resolve command based on type (npx, pip, cargo, etc.)
        let (cmd, args) = resolve_spawn_command(&config.command.join(" "))?;

        let mut command = Command::new(&cmd);
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
            match McpServer::spawn(name, mcp_config).await {
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

/// Helper to parse command format like "npx some-tool@latest" or "pip install some-pkg"
/// Handles quoted arguments (e.g., `"arg with spaces"`) and ensures proper splitting.
pub(crate) fn resolve_spawn_command(command: &str) -> Result<(String, Vec<String>), String> {
    let parts = shell_words::split(command).map_err(|e| e.to_string())?;
    if parts.is_empty() {
        return Err("Empty command".into());
    }
    let bin = parts[0].to_string();
    let args = parts[1..].to_vec();
    Ok((bin, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_spawn_command_npx() {
        let (cmd, args) = resolve_spawn_command("npx some-tool@latest").unwrap();
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["some-tool@latest"]);
    }

    #[test]
    fn test_resolve_spawn_command_simple() {
        let (cmd, args) = resolve_spawn_command("ls").unwrap();
        assert_eq!(cmd, "ls");
        assert!(args.is_empty());
    }

    #[test]
    fn test_resolve_spawn_command_empty() {
        assert!(resolve_spawn_command("").is_err());
    }
}
