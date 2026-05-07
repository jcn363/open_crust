use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;

/// A generic JSON-RPC 2.0 client over stdio
pub struct JsonRpcClient {
    child: Child,
    id_counter: u64,
}

impl JsonRpcClient {
    pub fn new(child: Child) -> Self {
        Self {
            child,
            id_counter: 0,
        }
    }

    /// Make a JSON-RPC call and return the result
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

        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or("No stdin for JSON-RPC server")?;
        stdin
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;

        let stdout = self
            .child
            .stdout
            .as_mut()
            .ok_or("No stdout for JSON-RPC server")?;
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
            if let Ok(response) = serde_json::from_str::<Value>(&line)
                && response.get("id").and_then(|v| v.as_u64()) == Some(id)
            {
                if let Some(error) = response.get("error") {
                    return Err(format!("JSON-RPC Error: {}", error));
                }
                return Ok(response.get("result").cloned().unwrap_or(Value::Null));
            }
        }

        Err("JSON-RPC server closed stdout unexpectedly".into())
    }

    /// Send a JSON-RPC notification (no response expected)
    pub async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let mut notif_str = serde_json::to_string(&notification).map_err(|e| e.to_string())?;
        notif_str.push('\n');

        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or("No stdin for JSON-RPC server")?;
        stdin
            .write_all(notif_str.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
