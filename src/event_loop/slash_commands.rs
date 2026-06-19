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

    // --- External Editor ---
    if prompt_str.starts_with("/edit ") {
        let file_path = prompt_str.trim_start_matches("/edit ").trim();
        if file_path.is_empty() {
            let _ = response_tx
                .send(
                    "opencrust: Usage: /edit <file_path> (opens file in external editor)"
                        .to_string(),
                )
                .await;
        } else if let Some(editor) = client.config.external_editor.as_deref() {
            let _ = response_tx
                .send(format!(
                    "opencrust: Opening '{}' in {}...",
                    file_path, editor
                ))
                .await;
            match std::process::Command::new(editor).arg(file_path).spawn() {
                Ok(_) => {
                    let _ = response_tx
                        .send(format!(
                            "opencrust: Launched {} for '{}'",
                            editor, file_path
                        ))
                        .await;
                }
                Err(e) => {
                    let _ = response_tx
                        .send(format!("opencrust: Failed to launch {}: {}", editor, e))
                        .await;
                }
            }
        } else {
            let _ = response_tx
                .send(
                    "opencrust: No external editor configured. Set 'external_editor' in config.json (e.g., \"external_editor\": \"code\")".to_string()
                )
                .await;
        }
        return true;
    }

    // --- Split View / Diff ---
    if prompt_str.starts_with("/diff ") {
        let file_path = prompt_str.trim_start_matches("/diff ").trim();
        if file_path.is_empty() {
            let _ = response_tx
                .send("opencrust: Usage: /diff <file_path> (shows file in split view)".to_string())
                .await;
        } else {
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    let _ = response_tx
                        .send(format!(
                            "opencrust: Loaded '{}' ({} bytes) into split view. Press Ctrl+D to enter split view.",
                            file_path, content.len()
                        ))
                        .await;
                }
                Err(e) => {
                    let _ = response_tx
                        .send(format!("opencrust: Error reading '{}': {}", file_path, e))
                        .await;
                }
            }
        }
        return true;
    }

    // --- Memory ---
    if prompt_str.starts_with("/memory ") {
        let args = prompt_str.trim_start_matches("/memory ").trim();
        if args.starts_with("remember ") {
            let rest = args.trim_start_matches("remember ").trim();
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                let _ = response_tx
                    .send("opencrust: Usage: /memory remember <key> <value>".to_string())
                    .await;
            } else {
                let mut mem = crate::memory::AutoMemory::default();
                mem.remember(
                    parts[0],
                    parts[1],
                    crate::memory::MemoryCategory::UserPreference,
                );
                let _ = response_tx
                    .send(format!(
                        "opencrust: Remembered '{}': '{}'",
                        parts[0], parts[1]
                    ))
                    .await;
            }
        } else if args.starts_with("recall ") {
            let query = args.trim_start_matches("recall ").trim();
            let mut mem = crate::memory::AutoMemory::default();
            let results = mem.recall(query);
            if results.is_empty() {
                let _ = response_tx
                    .send(format!("opencrust: No memories matching '{}'", query))
                    .await;
            } else {
                let list: Vec<String> = results
                    .iter()
                    .map(|e| format!("  [{}] {}: {}", e.category, e.key, e.value))
                    .collect();
                let _ = response_tx
                    .send(format!(
                        "opencrust: Found {} memories:\n{}",
                        results.len(),
                        list.join("\n")
                    ))
                    .await;
            }
        } else if args.starts_with("forget ") {
            let key = args.trim_start_matches("forget ").trim();
            let mut mem = crate::memory::AutoMemory::default();
            if mem.forget(key) {
                let _ = response_tx
                    .send(format!("opencrust: Forgot memory '{}'", key))
                    .await;
            } else {
                let _ = response_tx
                    .send(format!("opencrust: No memory with key '{}'", key))
                    .await;
            }
        } else if args == "list" {
            let mem = crate::memory::AutoMemory::default();
            let all = mem.list_all();
            if all.is_empty() {
                let _ = response_tx
                    .send("opencrust: No memories stored.".to_string())
                    .await;
            } else {
                let list: Vec<String> = all
                    .iter()
                    .map(|e| format!("  [{}] {}: {}", e.category, e.key, e.value))
                    .collect();
                let _ = response_tx
                    .send(format!(
                        "opencrust: {} memories:\n{}",
                        all.len(),
                        list.join("\n")
                    ))
                    .await;
            }
        } else {
            let _ = response_tx
                .send("opencrust: Usage: /memory <remember|recall|forget|list> [args]".to_string())
                .await;
        }
        return true;
    }

    // --- Agent (Recursive Sub-agents) ---
    if prompt_str.starts_with("/agent ") {
        let args = prompt_str.trim_start_matches("/agent ").trim();
        if args.starts_with("spawn ") {
            let prompt = args.trim_start_matches("spawn ").trim();
            let mut mgr = crate::recursive_agents::RecursiveAgentManager::new();
            match mgr.spawn_agent(None, prompt) {
                Ok(id) => {
                    let _ = response_tx
                        .send(format!(
                            "opencrust: Spawned agent '{}' (id: {}). Use Ctrl+T to view background tasks.",
                            &prompt[..40.min(prompt.len())],
                            &id[..12.min(id.len())]
                        ))
                        .await;
                }
                Err(e) => {
                    let _ = response_tx
                        .send(format!("opencrust: Failed to spawn agent: {}", e))
                        .await;
                }
            }
        } else if args == "status" {
            let mgr = crate::recursive_agents::RecursiveAgentManager::new();
            let count = mgr.get_total_count();
            let _ = response_tx
                .send(format!(
                    "opencrust: {} total agents. Use Ctrl+T for full background task view.",
                    count
                ))
                .await;
        } else if args == "tree" {
            let mgr = crate::recursive_agents::RecursiveAgentManager::new();
            let tree = mgr.render_tree();
            if tree.is_empty() {
                let _ = response_tx
                    .send("opencrust: No agents running.".to_string())
                    .await;
            } else {
                let _ = response_tx
                    .send(format!("opencrust: Agent tree:\n{}", tree.join("\n")))
                    .await;
            }
        } else {
            let _ = response_tx
                .send("opencrust: Usage: /agent <spawn|status|tree> [args]".to_string())
                .await;
        }
        return true;
    }

    // --- Auth (GitHub Copilot / ChatGPT Plus login) ---
    if prompt_str.starts_with("/auth ") {
        let args = prompt_str.trim_start_matches("/auth ").trim();
        if args == "copilot" {
            let _ = response_tx
                .send("opencrust: Initiating GitHub Copilot device flow... Visit https://github.com/login/device".to_string())
                .await;
            match crate::auth::github_copilot_device_flow().await {
                Ok(flow) => {
                    let _ = response_tx
                        .send(format!(
                            "opencrust: Go to {} and enter code: {}",
                            flow.verification_uri, flow.user_code
                        ))
                        .await;
                }
                Err(e) => {
                    let _ = response_tx
                        .send(format!("opencrust: Copilot auth error: {}", e))
                        .await;
                }
            }
        } else if args == "chatgpt" {
            let _ = response_tx
                .send("opencrust: Initiating ChatGPT Plus device flow...".to_string())
                .await;
            match crate::auth::chatgpt_plus_device_flow().await {
                Ok(flow) => {
                    let _ = response_tx
                        .send(format!(
                            "opencrust: Go to {} and enter code: {}",
                            flow.verification_uri, flow.user_code
                        ))
                        .await;
                }
                Err(e) => {
                    let _ = response_tx
                        .send(format!("opencrust: ChatGPT auth error: {}", e))
                        .await;
                }
            }
        } else if args == "status" {
            let copilot = crate::auth::is_authenticated(&crate::auth::AuthProvider::GitHubCopilot);
            let chatgpt = crate::auth::is_authenticated(&crate::auth::AuthProvider::ChatGptPlus);
            let _ = response_tx
                .send(format!(
                    "opencrust: Auth status — GitHub Copilot: {} | ChatGPT Plus: {}",
                    if copilot { "✓" } else { "✗" },
                    if chatgpt { "✓" } else { "✗" }
                ))
                .await;
        } else if args == "clear" {
            let _ = crate::auth::clear_token(&crate::auth::AuthProvider::GitHubCopilot);
            let _ = crate::auth::clear_token(&crate::auth::AuthProvider::ChatGptPlus);
            let _ = response_tx
                .send("opencrust: All auth tokens cleared.".to_string())
                .await;
        } else {
            let _ = response_tx
                .send("opencrust: Usage: /auth <copilot|chatgpt|status|clear>".to_string())
                .await;
        }
        return true;
    }

    // --- Share ---
    if prompt_str == "/share" {
        let _ = response_tx
            .send(
                "opencrust: Share link generation requires App context. Use /share-file instead."
                    .to_string(),
            )
            .await;
        return true;
    }

    if prompt_str == "/share-list" {
        let links = crate::event_loop::share::list_share_links();
        if links.is_empty() {
            let _ = response_tx
                .send("opencrust: No share links found.".to_string())
                .await;
        } else {
            let list: Vec<String> = links
                .iter()
                .map(|l| {
                    format!(
                        "  {} | {} msgs | {} {} | {}",
                        &l.id[..12.min(l.id.len())],
                        l.message_count,
                        l.provider,
                        l.model,
                        &l.created_at[..10.min(l.created_at.len())]
                    )
                })
                .collect();
            let _ = response_tx
                .send(format!(
                    "opencrust: {} share links:\n{}",
                    links.len(),
                    list.join("\n")
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
