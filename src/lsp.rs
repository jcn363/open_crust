use std::process::Stdio;
use tokio::process::Command;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::config::LspConfig;
use crate::jsonrpc::JsonRpcClient;
use std::env;

pub struct LspServer {
    rpc: JsonRpcClient,
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
        let mut rpc = JsonRpcClient::new(child);

        // Perform LSP initialization
        Self::initialize(&mut rpc).await?;

        Ok(Self { rpc })
    }

    async fn initialize(rpc: &mut JsonRpcClient) -> Result<(), String> {
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

        rpc.call("initialize", params).await?;
        rpc.notify("initialized", json!({})).await?;
        Ok(())
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.rpc.call(method, params).await
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
        // TODO: we'd check extensions.
        self.servers.values_mut().next().ok_or("No LSP server available".to_string())
    }
}
