use crate::{config, context, git, llm, rules};
use serde_json::Value;
use tokio::sync::mpsc;

/// Spawn the LLM background task that handles prompts and commands
#[allow(dead_code)]
pub fn spawn_llm_task(
    llm_client: llm::LlmClient,
    mut prompt_rx: mpsc::Receiver<String>,
    response_tx: mpsc::Sender<String>,
    mut approval_rx: mpsc::Receiver<bool>,
) {
    tokio::spawn(async move {
        let mut messages_history: Vec<Value> = Vec::new();
        while let Some(prompt) = prompt_rx.recv().await {
            let prompt_str = prompt.trim();

            if prompt_str == "/init" {
                match rules::init_project_rules() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("opencrust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/provider ") {
                let new_provider = prompt_str.trim_start_matches("/provider ").trim();
                let mut new_config = (*llm_client.config).clone();
                match new_provider.to_lowercase().as_str() {
                    "ollama" => {
                        new_config.provider = config::ProviderType::Ollama;
                        new_config.save();
                        let _ = response_tx
                            .send("opencrust: Provider switched to Ollama".to_string())
                            .await;
                    }
                    "openrouter" => {
                        new_config.provider = config::ProviderType::OpenRouter;
                        new_config.save();
                        let _ = response_tx
                            .send("opencrust: Provider switched to OpenRouter".to_string())
                            .await;
                    }
                    _ => {
                        let _ = response_tx
                            .send(format!("opencrust: Unknown provider '{}'", new_provider))
                            .await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/model ") {
                let new_model = prompt_str.trim_start_matches("/model ").trim();
                let mut new_config = (*llm_client.config).clone();
                new_config.model = new_model.to_string();
                new_config.save();
                let _ = response_tx
                    .send(format!("opencrust: Model switched to '{}'", new_model))
                    .await;
                continue;
            } else if prompt_str == "/undo" {
                match git::undo() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("opencrust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            } else if prompt_str == "/redo" {
                match git::redo() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("opencrust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/goal ") {
                let goal_desc = prompt_str.trim_start_matches("/goal ").trim();
                if goal_desc.is_empty() {
                    let _ = response_tx
                        .send("opencrust: Usage: /goal <description>".to_string())
                        .await;
                } else {
                    llm_client.set_goal(goal_desc.to_string());
                    let _ = response_tx
                        .send(format!(
                            "opencrust: Goal set: '{}'. Agent will work autonomously until completed. Use /goal-clear to reset.",
                            goal_desc
                        ))
                        .await;
                }
                continue;
            } else if prompt_str == "/goal-clear" || prompt_str == "/goal clear" {
                llm_client.clear_goal();
                let _ = response_tx
                    .send("opencrust: Goal cleared.".to_string())
                    .await;
                continue;
            } else if prompt_str == "/goal-status" || prompt_str == "/goal status" {
                match llm_client.get_goal() {
                    Some(goal) => {
                        let _ = response_tx
                            .send(format!(
                                "opencrust: Active goal: '{}' (set {})",
                                goal.description,
                                goal.created_at.format("%Y-%m-%d %H:%M")
                            ))
                            .await;
                    }
                    None => {
                        let _ = response_tx
                            .send("opencrust: No active goal.".to_string())
                            .await;
                    }
                }
                continue;
            }

            let _ = git::checkpoint();
            let enriched_prompt = context::inject_file_context(&prompt);
            let _ = response_tx
                .send(String::from("opencrust: Thinking..."))
                .await;

            let res = llm_client
                .send_message(
                    &mut messages_history,
                    &enriched_prompt,
                    response_tx.clone(),
                    Some(&mut approval_rx),
                )
                .await;
            match res {
                Ok(reply) => {
                    let _ = response_tx.send(format!("opencrust: {}", reply)).await;
                }
                Err(e) => {
                    let _ = response_tx.send(format!("Error: {}", e)).await;
                }
            }
        }
    });
}
