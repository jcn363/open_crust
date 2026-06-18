//! LLM client core — provider abstraction and tool-calling loop
//!
//! Wraps multiple LLM providers (OpenAI, Anthropic, Gemini, Ollama, OpenRouter,
//! Mistral, etc.) behind a unified interface. Manages the agentic loop: sends
//! messages, handles tool calls, enforces permissions, records audit logs, and
//! supports streaming responses. Central orchestrator for all LLM interactions.

mod completion;
mod context;
mod goals;
mod plan_mode;
pub(crate) mod providers;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::{Goal, PlanModeState};

use crate::audit::AuditLogger;
use crate::config::{Config, PermissionAction};
use crate::custom_tools::CustomToolManager;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::orchestrator::Orchestrator;
use crate::permissions::PermissionManager;
use crate::planner::Planner;
use crate::plugins::PluginManager;
use crate::rag::RagManager;
use crate::skills::SkillManager;
use crate::token_budget::TokenBudgetManager;
use crate::tool_executor::ToolExecutor;
use crate::web::WebManager;
use async_recursion::async_recursion;
use reqwest::Client;
use serde_json::{Value, json};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

/// Unified LLM client: provider abstraction and tool-calling loop
#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    pub config: Arc<Config>,
    pub mcp_manager: Arc<Mutex<McpManager>>,
    pub skill_manager: Arc<Mutex<SkillManager>>,
    pub custom_tool_manager: Arc<Mutex<CustomToolManager>>,
    pub permission_manager: Arc<PermissionManager>,
    pub audit_logger: Arc<AuditLogger>,
    pub pinned_files: Arc<Mutex<Vec<String>>>,
    pub tool_executor: Arc<ToolExecutor>,
    pub token_budget_manager: Arc<TokenBudgetManager>,
    pub plugin_manager: Arc<Mutex<PluginManager>>,
    plan_mode: Arc<std::sync::Mutex<PlanModeState>>,
    goal: Arc<std::sync::Mutex<Option<Goal>>>,
    pub orchestrator_tasks: Arc<tokio::sync::RwLock<Vec<crate::orchestrator::task::Task>>>,
}

impl LlmClient {
    pub fn new(
        config: Arc<Config>,
        mcp_manager: Arc<Mutex<McpManager>>,
        lsp_manager: Arc<Mutex<LspManager>>,
        skill_manager: Arc<Mutex<SkillManager>>,
        custom_tool_manager: Arc<Mutex<CustomToolManager>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let permission_manager = Arc::new(PermissionManager::new(config.clone()));
        let audit_logger = Arc::new(AuditLogger::new());
        let pinned_files = Arc::new(Mutex::new(Vec::new()));

        let orchestrator_tasks: Arc<tokio::sync::RwLock<Vec<crate::orchestrator::task::Task>>> =
            Arc::new(tokio::sync::RwLock::new(Vec::new()));

        let orchestrator = Arc::new(Mutex::new(
            Orchestrator::new(config.clone()).with_shared_state(orchestrator_tasks.clone()),
        ));

        let mut plugin_mgr = PluginManager::new();
        if config.plugins.enabled {
            let discovered = plugin_mgr.discover();
            plugin_mgr.load_state();
            for disabled in &config.plugins.disabled_plugins {
                let _ = plugin_mgr.disable(disabled);
            }
            if !discovered.is_empty() {
                eprintln!("[Plugins] Discovered: {}", discovered.join(", "));
            }
            let results = plugin_mgr.execute_hook("on_startup", "{}");
            for (name, result) in &results {
                match result {
                    Ok(output) => eprintln!("[Plugin:{}] on_startup: {}", name, output.trim()),
                    Err(e) => eprintln!("[Plugin:{}] on_startup error: {}", name, e),
                }
            }
        }
        let plugin_manager = Arc::new(Mutex::new(plugin_mgr));

        let tool_executor = Arc::new(ToolExecutor::new(
            config.clone(),
            mcp_manager.clone(),
            lsp_manager.clone(),
            skill_manager.clone(),
            custom_tool_manager.clone(),
            permission_manager.clone(),
            Arc::new(WebManager::new()?),
            Arc::new(Mutex::new(Planner::new())),
            Arc::new(Mutex::new(RagManager::new(&config))),
            pinned_files.clone(),
            orchestrator.clone(),
            plugin_manager.clone(),
        ));

        Ok(Self {
            client: Client::new(),
            config,
            mcp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
            audit_logger,
            pinned_files,
            tool_executor,
            token_budget_manager: Arc::new(TokenBudgetManager::new()),
            plugin_manager,
            plan_mode: Arc::new(std::sync::Mutex::new(PlanModeState::default())),
            goal: Arc::new(std::sync::Mutex::new(None)),
            orchestrator_tasks,
        })
    }

    /// Ensure a token budget exists for the given session, creating one if needed.
    pub async fn ensure_budget(&self, session_id: &str, max_tokens: u32) {
        if self
            .token_budget_manager
            .get_budget(session_id)
            .await
            .is_none()
        {
            self.token_budget_manager
                .create_budget(session_id.to_string(), max_tokens)
                .await;
        }
    }

    /// Record token usage for the current session.
    pub async fn record_usage(&self, session_id: &str, usage: crate::token_budget::TokenUsage) {
        self.token_budget_manager.add_usage(session_id, usage).await;
    }

    /// Check if the session has exceeded its token budget.
    pub async fn is_over_budget(&self, session_id: &str) -> bool {
        self.token_budget_manager.is_over_budget(session_id).await
    }

    #[async_recursion]
    pub async fn send_message(
        &self,
        messages_history: &mut Vec<Value>,
        prompt: &str,
        progress_tx: mpsc::Sender<String>,
        mut approval_rx: Option<&mut mpsc::Receiver<bool>>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Security: check for prompt injection attempts
        if let Err(e) = crate::security::check_prompt_injection(prompt) {
            return Err(format!("Security: {}", e).into());
        }

        // Fire on_message hook
        {
            let plugins = self.plugin_manager.lock().await;
            let hook_ctx = serde_json::json!({"prompt": prompt});
            let results = plugins.execute_hook(
                "on_message",
                &serde_json::to_string(&hook_ctx).unwrap_or_default(),
            );
            for (plugin_name, result) in &results {
                if let Err(e) = result {
                    eprintln!("[Plugin:{}] on_message error: {}", plugin_name, e);
                }
            }
        }

        // Build system prompt on first message
        if messages_history.is_empty() {
            let system_prompt = self.build_system_prompt().await;
            messages_history.push(json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        messages_history.push(json!({
            "role": "user",
            "content": prompt
        }));

        // Auto-summarize or prune old context
        let (summarized, summary) = self.check_and_summarize_context(messages_history).await;
        if summarized {
            if let Some(s) = summary {
                let _ = progress_tx
                    .send(format!("opencrust: Summarized old context: {}", s))
                    .await;
            }
        } else if messages_history.len() > 22 {
            let _ = progress_tx
                .send(String::from(
                    "opencrust: Pruning old context to stay within limits...",
                ))
                .await;
            let system_prompt = messages_history.remove(0);
            let last_messages = messages_history.split_off(messages_history.len() - 20);
            *messages_history = vec![system_prompt];
            messages_history.extend(last_messages);
        }

        // Agentic tool-calling loop
        loop {
            let res = self
                .dispatch_with_fallback(messages_history, None, &progress_tx)
                .await?;

            // Track token usage from response
            {
                let usage = crate::token_budget::extract_usage_from_response(
                    &self.config.provider.to_string(),
                    &res,
                );
                if usage.total_tokens > 0 {
                    // Use a session-level budget key derived from config
                    let budget_key = format!("{}:{}", self.config.provider, self.config.model);
                    self.record_usage(&budget_key, usage).await;
                }
            }

            if let Some(tool_calls) = res.get("tool_calls").and_then(|t| t.as_array()) {
                messages_history.push(res.clone());

                for tool_call in tool_calls {
                    let id = tool_call.get("id").and_then(|i| i.as_str()).unwrap_or("");
                    let name = tool_call
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let args_str = tool_call
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|a| a.as_str())
                        .unwrap_or("{}");
                    let args: Value = serde_json::from_str(args_str).unwrap_or(json!({}));

                    let input_summary = match name {
                        "bash" => args.get("command").and_then(|v| v.as_str()).unwrap_or(""),
                        "write" => args.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        "read" => args.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        _ => args_str,
                    };

                    // Permission check
                    let mut approved = self
                        .permission_manager
                        .is_allowed_without_prompt(name, input_summary);
                    if !approved {
                        let permission = self
                            .permission_manager
                            .check_permission(name, input_summary);
                        if permission == PermissionAction::Ask {
                            if name == "write" {
                                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                let proposed =
                                    args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                let original = std::fs::read_to_string(path).unwrap_or_default();
                                let _ = progress_tx
                                    .send(format!(
                                        "[DIFF_REQUIRED]{}|{}|{}",
                                        path, original, proposed
                                    ))
                                    .await;
                            } else {
                                let _ = progress_tx.send(format!("opencrust: [APPROVAL_REQUIRED] The agent wants to run '{}' with input: '{}'. Allow? (y/n)", name, input_summary)).await;
                            }
                            approved = match approval_rx.as_deref_mut() {
                                Some(rx) => rx.recv().await.unwrap_or(false),
                                None => true,
                            };
                        } else {
                            approved = false;
                        }
                    }

                    self.audit_logger.log_action(name, input_summary, approved);

                    let plan_blocked = self.is_tool_blocked_in_plan_mode(name);
                    if plan_blocked {
                        let _ = progress_tx
                            .send(format!(
                                "opencrust: [PLAN MODE] Tool '{}' blocked — switch to execution mode to allow changes.",
                                name
                            ))
                            .await;
                    }

                    let result = if approved && !plan_blocked {
                        let _ = progress_tx
                            .send(format!("opencrust: Executing tool '{}'...", name))
                            .await;

                        match self.tool_executor.execute(name, &args).await {
                            Ok(res) => res,
                            Err(e) => {
                                // Self-healing: structured error with original args
                                // lets the LLM retry with corrected parameters
                                format!(
                                    "Tool '{}' failed: {}\n\nOriginal arguments: {}\n\n{}",
                                    name, e, args_str,
                                    "Suggestions:\n- Check argument spelling and types\n- Ensure paths exist and are valid\n- Use correct parameter names shown in the tool schema"
                                )
                            }
                        }
                    } else {
                        let _ = progress_tx
                            .send(format!("opencrust: Tool '{}' denied by user.", name))
                            .await;
                        String::from("User denied permission to execute this tool.")
                    };

                    messages_history.push(json!({
                        "role": "tool",
                        "tool_call_id": id,
                        "name": name,
                        "content": result
                    }));
                }
            } else if let Some(content) = res.get("content").and_then(|c| c.as_str()) {
                messages_history.push(res.clone());
                // Fire on_response hook
                {
                    let plugins = self.plugin_manager.lock().await;
                    let hook_ctx = serde_json::json!({"response_length": content.len()});
                    let results = plugins.execute_hook(
                        "on_response",
                        &serde_json::to_string(&hook_ctx).unwrap_or_default(),
                    );
                    for (plugin_name, result) in &results {
                        if let Err(e) = result {
                            eprintln!("[Plugin:{}] on_response error: {}", plugin_name, e);
                        }
                    }
                }
                return Ok(content.to_string());
            } else {
                return Err("Empty response".into());
            }
        }
    }
}

/// Create a minimal LlmClient for testing.
#[cfg(test)]
pub(crate) fn new_test_client(
    config: Arc<Config>,
) -> Result<LlmClient, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::sync::Mutex;
    let mcp = Arc::new(Mutex::new(crate::mcp::McpManager::new()));
    let lsp = Arc::new(Mutex::new(crate::lsp::LspManager::new()));
    let skills = Arc::new(Mutex::new(crate::skills::SkillManager::new()));
    let custom = Arc::new(Mutex::new(crate::custom_tools::CustomToolManager::new()));
    LlmClient::new(config, mcp, lsp, skills, custom)
}
