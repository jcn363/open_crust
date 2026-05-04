use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::config::LspConfig;
use std::env;

pub struct LspServer {
    pub name: String,
    child: Child,
    id_counter: u64,
}

impl LspServer {
    pub async fn spawn(name: &str, config: &LspConfig) -> Result<Self, String> {
        if config.command.is_empty() {
            return Err("Empty LSP command".into());
        }

        let mut cmd = Command::new(&config.command[0]);
        if config.command.len() > 1 {
            cmd.args(&config.command[1..]);
        }

        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::inherit());

        if let Some(env_vars) = &config.env {
            for (k, v) in env_vars {
                cmd.env(k, v);
            }
        }

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn LSP server {}: {}", name, e))?;

        let mut server = Self {
            name: name.to_string(),
            child,
            id_counter: 0,
        };

        // Perform LSP initialization
        server.initialize().await?;

        Ok(server)
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let root_dir = env::current_dir().unwrap_or_default();
        let root_uri = format!("file://{}", root_dir.display());

        let params = json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "definition": { "dynamicRegistration": true },
                    "references": { "dynamicRegistration": true },
                    "hover": { "dynamicRegistration": true }
                }
            }
        });

        self.call("initialize", params).await?;
        self.notify("initialized", json!({})).await?;
        Ok(())
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

        let stdin = self.child.stdin.as_mut().ok_or("No stdin for LSP server")?;
        stdin.write_all(request_str.as_bytes()).await.map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;

        let stdout = self.child.stdout.as_mut().ok_or("No stdout for LSP server")?;
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
            // LSP uses Content-Length headers, but many simple servers also just emit JSON lines.
            // For a robust implementation, we'd need to parse the headers.
            // Here we assume standard JSON-RPC over stdio.
            if let Ok(response) = serde_json::from_str::<Value>(&line) {
                if response.get("id").and_then(|v| v.as_u64()) == Some(id) {
                    if let Some(error) = response.get("error") {
                        return Err(format!("LSP Error: {}", error));
                    }
                    return Ok(response.get("result").cloned().unwrap_or(Value::Null));
                }
            }
        }

        Err("LSP server closed stdout unexpectedly".into())
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let mut request_str = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        request_str.push('\n');

        let stdin = self.child.stdin.as_mut().ok_or("No stdin for LSP server")?;
        stdin.write_all(request_str.as_bytes()).await.map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub struct LspManager {
    pub servers: HashMap<String, LspServer>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub async fn load_from_config(&mut self, lsp_configs: &HashMap<String, LspConfig>) {
        for (name, config) in lsp_configs {
            if !config.disabled {
                match LspServer::spawn(name, config).await {
                    Ok(server) => {
                        self.servers.insert(name.clone(), server);
                        println!("open_crust: LSP server '{}' connected.", name);
                    }
                    Err(e) => {
                        eprintln!("open_crust: Error starting LSP server '{}': {}", name, e);
                    }
                }
            }
        }
    }

    pub async fn goto_definition(&mut self, path: &str, line: u32, character: u32) -> Result<String, String> {
        self.call_lsp("textDocument/definition", path, line, character).await
    }

    pub async fn hover(&mut self, path: &str, line: u32, character: u32) -> Result<String, String> {
        self.call_lsp("textDocument/hover", path, line, character).await
    }

    pub async fn find_references(&mut self, path: &str, line: u32, character: u32) -> Result<String, String> {
        self.call_lsp("textDocument/references", path, line, character).await
    }

    pub async fn type_definition(&mut self, path: &str, line: u32, character: u32) -> Result<String, String> {
        self.call_lsp("textDocument/typeDefinition", path, line, character).await
    }

    async fn call_lsp(&mut self, method: &str, path: &str, line: u32, character: u32) -> Result<String, String> {
        let server = self.find_server_for_path(path)?;
        let uri = format!("file://{}", env::current_dir().unwrap().join(path).display());
        
        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        });

        let result = server.call(method, params).await?;
        Ok(format!("{}: {}", method, result))
    }

    fn find_server_for_path(&mut self, _path: &str) -> Result<&mut LspServer, String> {
        // For now, just return the first server if it exists.
        // In a real app, we'd check extensions.
        self.servers.values_mut().next().ok_or("No LSP server available".to_string())
    }
}
