use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::config::McpConfig;

pub struct McpServer {
    pub name: String,
    child: Child,
    id_counter: u64,
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

        Ok(Self {
            name: name.to_string(),
            child,
            id_counter: 0,
        })
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.id_counter += 1;
        let id = self.id_counter;

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let mut request_str = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        request_str.push('\n');

        let stdin = self.child.stdin.as_mut().ok_or("No stdin for MCP server")?;
        stdin.write_all(request_str.as_bytes()).await.map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;

        let stdout = self.child.stdout.as_mut().ok_or("No stdout for MCP server")?;
        let mut reader = BufReader::new(stdout).lines();

        if let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
            let response: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
            if response.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(error) = response.get("error") {
                    return Err(format!("MCP Error: {}", error));
                }
                return Ok(response.get("result").cloned().unwrap_or(Value::Null));
            } else {
                return Err("Mismatched MCP response ID".into());
            }
        }

        Err("MCP server closed stdout unexpectedly".into())
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
