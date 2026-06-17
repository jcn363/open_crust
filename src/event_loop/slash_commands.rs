//! Slash command handling for the TUI event loop.
//! Handles /init, /provider, /model, /undo, /redo, /goal, /goal-clear, /goal-status commands.

use crate::{context, git, llm, rules};
use serde_json::Value;
use tokio::sync::mpsc;

/// Handle slash commands entered by the user.
/// Returns `true` if a command was handled (caller should continue),
/// `false` if the input should be sent to the LLM.
pub(crate) async fn handle_slash_command(
    prompt_str: &str,
    client: &llm::LlmClient,
    response_tx: &mpsc::Sender<String>,
    messages_history: &mut Vec<Value>,
    approval_rx: &mut mpsc::Receiver<bool>,
) -> bool {
    if prompt_str == "/init" {
        match rules::init_project_rules() {
            Ok(msg) => {
                let _ = response_tx.send(format!("opencrust: {}", msg)).await;
            }
            Err(e) => {
                let _ = response_tx.send(format!("Error: {}", e)).await;
            }
        }
        return true;
    }

    if prompt_str.starts_with("/provider ") {
        let new_provider = prompt_str.trim_start_matches("/provider ").trim();
        let mut new_config = (*client.config).clone();
        match new_provider.to_lowercase().as_str() {
            "ollama" => {
                new_config.provider = crate::config::ProviderType::Ollama;
                new_config.save();
                let _ = response_tx
                    .send("opencrust: Provider switched to Ollama".to_string())
                    .await;
            }
            "openrouter" => {
                new_config.provider = crate::config::ProviderType::OpenRouter;
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
        return true;
    }

    if prompt_str.starts_with("/model ") {
        let new_model = prompt_str.trim_start_matches("/model ").trim();
        let mut new_config = (*client.config).clone();
        new_config.model = new_model.to_string();
        new_config.save();
        let _ = response_tx
            .send(format!("opencrust: Model switched to '{}'", new_model))
            .await;
        return true;
    }

    if prompt_str == "/undo" {
        match git::undo() {
            Ok(msg) => {
                let _ = response_tx.send(format!("opencrust: {}", msg)).await;
            }
            Err(e) => {
                let _ = response_tx.send(format!("Error: {}", e)).await;
            }
        }
        return true;
    }

    if prompt_str == "/redo" {
        match git::redo() {
            Ok(msg) => {
                let _ = response_tx.send(format!("opencrust: {}", msg)).await;
            }
            Err(e) => {
                let _ = response_tx.send(format!("Error: {}", e)).await;
            }
        }
        return true;
    }

    if prompt_str.starts_with("/goal ") {
        let goal_desc = prompt_str.trim_start_matches("/goal ").trim();
        if goal_desc.is_empty() {
            let _ = response_tx
                .send("opencrust: Usage: /goal <description>".to_string())
                .await;
        } else {
            client.set_goal(goal_desc.to_string());
            let _ = response_tx
                .send(format!(
                    "opencrust: Goal set: '{}'. Agent will work autonomously until completed. Use /goal-clear to reset.",
                    goal_desc
                ))
                .await;
        }
        return true;
    }

    if prompt_str == "/goal-clear" || prompt_str == "/goal clear" {
        client.clear_goal();
        let _ = response_tx
            .send("opencrust: Goal cleared.".to_string())
            .await;
        return true;
    }

    if prompt_str == "/goal-status" || prompt_str == "/goal status" {
        match client.get_goal() {
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
        return true;
    }

    if prompt_str == "/cost" {
        let budget_key = format!("{}:{}", client.config.provider, client.config.model);
        let budget = client.token_budget_manager.get_budget(&budget_key).await;
        match budget {
            Some(b) => {
                let pct = (b.usage_percentage() * 100.0) as u32;
                let status = if b.at_stop_threshold() {
                    "OVER BUDGET"
                } else if b.at_warning_threshold() {
                    "WARNING"
                } else {
                    "OK"
                };
                let _ = response_tx
                    .send(format!(
                        "opencrust: Session cost: ${:.4} | Tokens: {}/{} ({}%) | Status: {}",
                        b.total_cost, b.current_tokens, b.max_tokens, pct, status
                    ))
                    .await;
            }
            None => {
                let _ = response_tx
                    .send(format!(
                        "opencrust: No budget configured for {}:{}",
                        client.config.provider, client.config.model
                    ))
                    .await;
            }
        }
        return true;
    }

    if prompt_str.starts_with("/budget ") {
        let amount_str = prompt_str.trim_start_matches("/budget ").trim();
        match amount_str.parse::<u32>() {
            Ok(max_tokens) => {
                let budget_key = format!("{}:{}", client.config.provider, client.config.model);
                let budget = client
                    .token_budget_manager
                    .create_budget(budget_key.clone(), max_tokens)
                    .await;
                let _ = response_tx
                    .send(format!(
                        "opencrust: Budget set to {} tokens for {}:{}",
                        budget.max_tokens, client.config.provider, client.config.model
                    ))
                    .await;
            }
            Err(_) => {
                let _ = response_tx
                    .send(
                        "opencrust: Usage: /budget <max_tokens> (e.g., /budget 1000000)"
                            .to_string(),
                    )
                    .await;
            }
        }
        return true;
    }

    if prompt_str.starts_with("/fallback ") {
        let chain_str = prompt_str.trim_start_matches("/fallback ").trim();
        if chain_str.is_empty() || chain_str == "clear" {
            let mut new_config = (*client.config).clone();
            new_config.fallback_chain = Vec::new();
            new_config.save();
            let _ = response_tx
                .send("opencrust: Fallback chain cleared.".to_string())
                .await;
        } else {
            let chain: Vec<String> = chain_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
            let mut new_config = (*client.config).clone();
            new_config.fallback_chain = chain.clone();
            new_config.save();
            let _ = response_tx
                .send(format!(
                    "opencrust: Fallback chain set: {}",
                    chain.join(" → ")
                ))
                .await;
        }
        return true;
    }

    // Not a slash command - send to LLM
    let _ = git::checkpoint();
    let enriched_prompt = context::inject_file_context(prompt_str);
    let _ = response_tx
        .send(String::from("opencrust: Thinking..."))
        .await;

    let res = client
        .send_message(
            messages_history,
            &enriched_prompt,
            response_tx.clone(),
            Some(approval_rx),
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

    true
}
