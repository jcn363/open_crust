use crate::config::McpConfig;
use crate::jsonrpc::JsonRpcClient;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::fs;
use std::io::Write;
use tokio::process::Command;

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

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server {}: {}", name, e))?;
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
            if let Ok(result) = server.call("tools/list", json!({})).await
                && let Some(tools) = result.get("tools").and_then(|t| t.as_array())
            {
                for tool in tools {
                    let mut tool_cloned = tool.clone();
                    // Prefix tool name with server name to avoid conflicts
                    if let Some(tool_name) = tool_cloned.get_mut("name")
                        && let Some(s) = tool_name.as_str()
                    {
                        *tool_name = json!(format!("{}_{}", name, s));
                    }
                    all_tools.push(tool_cloned);
                }
            }
        }
        all_tools
    }

    pub async fn call_tool(
        &mut self,
        full_name: &str,
        arguments: &Value,
    ) -> Result<String, String> {
        for (name, server) in &mut self.servers {
            let prefix = format!("{}_", name);
            if full_name.starts_with(&prefix) {
                let tool_name = &full_name[prefix.len()..];
                let result = server
                    .call(
                        "tools/call",
                        json!({
                            "name": tool_name,
                            "arguments": arguments
                        }),
                    )
                    .await?;

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

/// Run OpenCrust as an MCP server, exposing its tools via MCP protocol
pub async fn run_mcp_server(port: u16, use_stdio: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting OpenCrust MCP Server...");
    
    if use_stdio {
        println!("Using stdio transport");
        run_stdio_server().await?;
    } else {
        println!("Using TCP transport on port {}", port);
        run_tcp_server(port).await?;
    }
    Ok(())
}

/// Run MCP server over stdio (JSON-RPC over stdin/stdout)
async fn run_stdio_server() -> Result<(), String> {
    println!("OpenCrust MCP Server ready (stdio)");
    
    // Load tools
    let tools = get_opencrust_tools();
    
    // Use a separate thread for blocking I/O
    let result = tokio::task::spawn_blocking(move || {
        use std::io::{BufRead, BufReader, Write};
        
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        
        while let Some(Ok(line)) = lines.next() {
            if line.trim().is_empty() {
                continue;
            }
            
            // Parse JSON-RPC request
            let request: Value = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    let _ = send_error_sync(&mut stdout, -32700, &format!("Parse error: {}", e));
                    continue;
                }
            };
            
            let id = request.get("id").cloned();
            let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            
            // Handle MCP methods
            let result = match method {
                "initialize" => {
                    json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "opencrust",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    })
                }
                "tools/list" => {
                    json!({
                        "tools": tools.iter().map(|t| json!({
                            "name": t.0,
                            "description": t.1,
                            "inputSchema": t.2
                        })).collect::<Vec<_>>()
                    })
                }
                "tools/call" => {
                    let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let arguments = params.get("arguments").unwrap_or(&Value::Null);
                    
                    // Execute tool (need to handle async in sync context)
                    let output = tokio::runtime::Handle::current()
                        .block_on(execute_opencrust_tool(tool_name, arguments));
                    
                    match output {
                        Ok(output) => json!({
                            "content": [{"type": "text", "text": output}]
                        }),
                        Err(e) => json!({
                            "isError": true,
                            "content": [{"type": "text", "text": e}]
                        })
                    }
                }
                _ => {
                    let _ = send_error_sync(&mut stdout, -32601, &format!("Method not found: {}", method));
                    continue;
                }
            };
            
            // Send response
            let response = json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            });
            
            if let Ok(response_str) = serde_json::to_string(&response) {
                let _ = stdout.write_all(response_str.as_bytes());
                let _ = stdout.write_all(b"\n");
                let _ = stdout.flush();
            }
        }
        
        Ok(())
    }).await;
    
        result.map_err(|e| format!("Stdio server error: {}", e))?
}

/// Run MCP server over TCP
async fn run_tcp_server(_port: u16) -> Result<(), String> {
    // For simplicity, we'll use a basic TCP listener
    // In production, you'd want proper async handling
    println!("TCP transport not yet implemented. Use --stdio flag.");
    Err("TCP transport not implemented".to_string())
}

/// Send a JSON-RPC error response (synchronous version for stdio)
fn send_error_sync(stdout: &mut std::io::Stdout, code: i32, message: &str) -> std::io::Result<()> {
    let error = json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {
            "code": code,
            "message": message
        }
    });
    
    if let Ok(error_str) = serde_json::to_string(&error) {
        stdout.write_all(error_str.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
    }
    Ok(())
}

/// Get list of OpenCrust tools exposed via MCP
fn get_opencrust_tools() -> Vec<(String, String, Value)> {
    vec![
        (
            "bash".to_string(),
            "Execute a bash command".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    }
                },
                "required": ["command"]
            })
        ),
        (
            "read".to_string(),
            "Read a file from the filesystem".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            })
        ),
        (
            "write".to_string(),
            "Write content to a file".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            })
        ),
    ]
}

/// Execute an OpenCrust tool
async fn execute_opencrust_tool(name: &str, arguments: &Value) -> Result<String, String> {
    match name {
        "bash" => {
            let command = arguments.get("command").and_then(|v| v.as_str()).unwrap_or("");
            match tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr))
                }
                Err(e) => Err(format!("Error executing command: {}", e)),
            }
        }
        "read" => {
            let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
            fs::read_to_string(path).map_err(|e| format!("Error reading file: {}", e))
        }
        "write" => {
            let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or("");
            fs::write(path, content).map_err(|e| format!("Error writing file: {}", e))?;
            
            // Format the file if possible
            crate::formatters::format_file(std::path::Path::new(path));
            
            Ok(format!("Successfully wrote to {}", path))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

