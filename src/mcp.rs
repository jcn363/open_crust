impl McpServer {
    pub async fn spawn(name: &str, config: &McpConfig) -> Result<Self, String> {
        if config.command.is_empty() {
            return Err("Empty MCP command".into());
        }

        // ✅ Resolve command based on type (npx, pip, cargo, etc.)
        let (cmd, args) = resolve_spawn_command(&config.command) ?; // Helper to parse npx/pip/cargo
        
        let mut command = Command::new(&cmd);
        command.args(args);
        
        // Configure stdio for JSON-RPC communication
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let process = command.spawn()?;
        let stdout = process.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = process.stderr.take().ok_or("Failed to capture stderr")?;
        let mut stdin = process.stdin.take().ok_or("Failed to capture stdin")?;

        // Initialize JSON-RPC client
        let rpc = JsonRpcClient::new(stdin, stdout, stderr);

        Ok(Self { rpc })
    }
}

/// Helper to parse command format like "npx some-tool@latest" or "pip install some-pkg"
fn resolve_spawn_command(command: &str) -> Result<(String, Vec<String>), String> {
    match command.split_whitespace().next() {
        Some("npx") | Some("pip") | Some("cargo") => {
            let bin = command.split_whitespace().nth(1).unwrap_or("");
            let args = command.split_whitespace().skip(2).map(|s| s.to_string()).collect();
            Ok((bin.to_string(), args))
        }
        _ => Ok((command.to_string(), Vec::new())),
    }
}