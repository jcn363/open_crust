//! LSP (Language Server Protocol) integration
//!
//! Manages LSP server child processes over JSON-RPC 2.0. Provides
//! language-aware code intelligence: diagnostics, completions, and
//! hover information for supported languages.

use crate::config::LspConfig;
use crate::jsonrpc::JsonRpcClient;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

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

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn LSP server {}: {}", name, e))?;
        let mut rpc = JsonRpcClient::new(child);

        // Perform LSP initialization
        Self::initialize(&mut rpc).await?;

        Ok(Self { rpc })
    }

    async fn initialize(rpc: &mut JsonRpcClient) -> Result<(), String> {
        let root_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
    configs: HashMap<String, LspConfig>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            configs: HashMap::new(),
        }
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LspManager {
    pub async fn add_server(&mut self, name: String, config: LspConfig) -> Result<(), String> {
        let server = LspServer::spawn(&name, &config).await?;
        self.servers.insert(name.clone(), server);
        self.configs.insert(name, config);
        Ok(())
    }

    #[expect(dead_code, reason = "runtime LSP server registration")]
    fn find_server_for_path(&self, path: &str) -> Result<&LspServer, String> {
        let path_ext = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Find server whose extensions match the file path
        for (name, config) in &self.configs {
            if (config.extensions.is_empty() || config.extensions.contains(&path_ext.to_string()))
                && let Some(server) = self.servers.get(name)
            {
                return Ok(server);
            }
        }

        Err("No LSP server available for this file type".to_string())
    }

    fn find_server_for_path_mut(&mut self, path: &str) -> Result<&mut LspServer, String> {
        let path_ext = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // First, find the server name that matches
        let mut server_name = None;
        for (name, config) in &self.configs {
            if (config.extensions.is_empty() || config.extensions.contains(&path_ext.to_string()))
                && self.servers.contains_key(name)
            {
                server_name = Some(name.clone());
                break;
            }
        }

        match server_name {
            Some(name) => self
                .servers
                .get_mut(&name)
                .ok_or_else(|| format!("LSP server '{}' was unregistered unexpectedly", name)),
            None => Err("No LSP server available for this file type".to_string()),
        }
    }

    pub async fn load_from_config(&mut self, lsp_configs: &HashMap<String, LspConfig>) {
        for (name, config) in lsp_configs {
            if !config.disabled {
                match LspServer::spawn(name, config).await {
                    Ok(server) => {
                        self.servers.insert(name.clone(), server);
                        self.configs.insert(name.clone(), config.clone());
                        println!("opencrust: LSP server '{}' connected.", name);
                    }
                    Err(e) => {
                        eprintln!("opencrust: Error starting LSP server '{}': {}", name, e);
                    }
                }
            }
        }
    }

    pub async fn goto_definition(
        &mut self,
        path: &str,
        line: u32,
        character: u32,
    ) -> Result<String, String> {
        self.call_lsp("textDocument/definition", path, line, character)
            .await
    }

    pub async fn hover(&mut self, path: &str, line: u32, character: u32) -> Result<String, String> {
        self.call_lsp("textDocument/hover", path, line, character)
            .await
    }

    pub async fn find_references(
        &mut self,
        path: &str,
        line: u32,
        character: u32,
    ) -> Result<String, String> {
        self.call_lsp("textDocument/references", path, line, character)
            .await
    }

    pub async fn type_definition(
        &mut self,
        path: &str,
        line: u32,
        character: u32,
    ) -> Result<String, String> {
        self.call_lsp("textDocument/typeDefinition", path, line, character)
            .await
    }

    pub async fn completion(
        &mut self,
        path: &str,
        line: u32,
        character: u32,
    ) -> Result<String, String> {
        let server = self.find_server_for_path_mut(path)?;
        let cwd = env::current_dir().unwrap_or_default();
        let uri = format!("file://{}", cwd.join(path).display());

        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        });

        let result = server.call("textDocument/completion", params).await?;
        Ok(format!("textDocument/completion: {}", result))
    }

    pub async fn diagnostics(&mut self, path: &str) -> Result<String, String> {
        let server = self.find_server_for_path_mut(path)?;
        let cwd = env::current_dir().unwrap_or_default();
        let uri = format!("file://{}", cwd.join(path).display());

        let params = json!({
            "textDocument": { "uri": uri }
        });

        let result = server.call("textDocument/diagnostic", params).await?;
        Ok(format!("textDocument/diagnostic: {}", result))
    }

    pub async fn formatting(&mut self, path: &str) -> Result<String, String> {
        let server = self.find_server_for_path_mut(path)?;
        let cwd = env::current_dir().unwrap_or_default();
        let uri = format!("file://{}", cwd.join(path).display());

        let params = json!({
            "textDocument": { "uri": uri },
            "options": {
                "tabSize": 4,
                "insertSpaces": true
            }
        });

        let result = server.call("textDocument/formatting", params).await?;
        Ok(format!("textDocument/formatting: {}", result))
    }

    async fn call_lsp(
        &mut self,
        method: &str,
        path: &str,
        line: u32,
        character: u32,
    ) -> Result<String, String> {
        let server = self.find_server_for_path_mut(path)?;
        let cwd = env::current_dir().unwrap_or_default();
        let uri = format!("file://{}", cwd.join(path).display());

        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        });

        let result = server.call(method, params).await?;
        Ok(format!("{}: {}", method, result))
    }
}
