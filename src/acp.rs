use std::io::{self, BufRead};
use serde_json::{json, Value};
use crate::llm::LlmClient;
use tokio::sync::mpsc;

pub async fn run_acp_loop(llm_client: LlmClient) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stdin = io::stdin();
    let reader = stdin.lock().lines();

    for line in reader {
        let line = line?;
        if let Ok(request) = serde_json::from_str::<Value>(&line) {
            let id = request.get("id").cloned();
            let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
            let params = request.get("params").cloned().unwrap_or(json!({}));

            match method {
                "chat" => {
                    let prompt = params.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
                    let mut history = params.get("messages").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    
                    let (progress_tx, mut _progress_rx) = mpsc::channel(32);
                    let (_approval_tx, mut approval_rx) = mpsc::channel(1);

                    // For ACP, we might need a non-interactive mode or auto-approval
                    // But for now, let's just run it.
                    
                    let client = llm_client.clone();
                    let response = client.send_message(&mut history, prompt, progress_tx, &mut approval_rx).await?;

                    let response_json = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "response": response,
                            "messages": history
                        }
                    });
                    println!("{}", response_json);
                }
                "initialize" => {
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "capabilities": {
                                "chat": true,
                                "applyChanges": true
                            }
                        }
                    });
                    println!("{}", response);
                }
                _ => {
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": "Method not found"
                        }
                    });
                    println!("{}", response);
                }
            }
        }
    }

    Ok(())
}
