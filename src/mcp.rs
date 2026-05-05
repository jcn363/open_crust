use std::process::Stdio;
use tokio::process::Command;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::config::McpConfig;
use crate::jsonrpc::JsonRpcClient;

pub struct McpServer {
    rpc: JsonRpcClient,
}

impl McpServer {
    pub async fn spawn(name: &str, config: &McpConfig) -> Result<Self, String> {
        if config.command.is_empty() {
            return Err("Empty MCP command".into());
        }

        let mut cmd = Command::new(&config.command[0]);
        if config.command.len() > 1 {
            cmd.args(&config.command[1..]);
        }

        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::inherit());

        if let Some(env) = &config.environment {
            for (k, v) in env {
                cmd.env(k, v);
            }
        }

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn MCP server {}: {}", name, e))?;
        let rpc = JsonRpcClient::new(child);

        Ok(Self { rpc })
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.rpc.call(method, params).await
    }
}

pub struct McpManager {
    pub servers: HashMap<String, McpServer>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub async fn load_from_config(&mut self, mcp_configs: &HashMap<String, McpConfig>) {
        for (name, config) in mcp_configs {
            if config.enabled {
                match McpServer::spawn(name, config).await {
                    Ok(server) => {
                        self.servers.insert(name.clone(), server);
                        println!("open_crust: MCP server '{}' connected.", name);
                    }
                    Err(e) => {
                        eprintln!("open_crust: Error starting MCP server '{}': {}", name, e);
                    }
                }
            }
        }
    }

    pub async fn list_tools(&mut self) -> Vec<Value> {
        let mut all_tools = Vec::new();
        for (name, server) in &mut self.servers {
            if let Ok(result) = server.call("tools/list", json!({})).await {
                if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                    for tool in tools {
                        let mut tool_cloned = tool.clone();
                        // Prefix tool name with server name to avoid conflicts
                        if let Some(tool_name) = tool_cloned.get_mut("name") {
                            if let Some(s) = tool_name.as_str() {
                                *tool_name = json!(format!("{}_{}", name, s));
                            }
                        }
                        all_tools.push(tool_cloned);
                    }
                }
            }
        }
        all_tools
    }

    pub async fn call_tool(&mut self, full_name: &str, arguments: &Value) -> Result<String, String> {
        for (name, server) in &mut self.servers {
            let prefix = format!("{}_", name);
            if full_name.starts_with(&prefix) {
                let tool_name = &full_name[prefix.len()..];
                let result = server.call("tools/call", json!({
                    "name": tool_name,
                    "arguments": arguments
                })).await?;
                
                // MCP tool results are usually a list of content items (text, image, etc.)
                if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                    let mut output = String::new();
                    for item in content {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            output.push_str(text);
                        }
                    }
                    return Ok(output);
                }
                return Ok(result.to_string());
            }
        }
        Err(format!("MCP tool '{}' not found", full_name))
    }
}
