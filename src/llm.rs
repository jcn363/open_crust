use crate::config::{Config, PermissionAction, ProviderType};
use crate::rules;
use crate::tool_executor::ToolExecutor;
use reqwest::Client;
use serde_json::{Value, json};
use std::error::Error;
use tokio::sync::mpsc;

const BASE_SYSTEM_PROMPT: &str = "You are open_crust, a pure Rust terminal-based AI coding agent. 
You have access to tools to interact with the local filesystem and execute bash commands.
Always follow the project's rules and guidelines provided below.";

use crate::audit::AuditLogger;
use crate::custom_tools::CustomToolManager;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::permissions::PermissionManager;
use crate::planner::Planner;
use crate::rag::RagManager;
use crate::skills::SkillManager;
use crate::stats::UsageStats;
use crate::web::WebManager;
use async_recursion::async_recursion;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    pub config: Config,
    pub mcp_manager: Arc<Mutex<McpManager>>,
    pub skill_manager: Arc<Mutex<SkillManager>>,
    pub custom_tool_manager: Arc<Mutex<CustomToolManager>>,
    pub permission_manager: Arc<PermissionManager>,
    pub audit_logger: Arc<AuditLogger>,
    pub usage_stats: Arc<Mutex<UsageStats>>,
    pub pinned_files: Arc<Mutex<Vec<String>>>,
    pub tool_executor: Arc<ToolExecutor>,
}

impl LlmClient {
    pub fn new(
        config: Config,
        mcp_manager: Arc<Mutex<McpManager>>,
        lsp_manager: Arc<Mutex<LspManager>>,
        skill_manager: Arc<Mutex<SkillManager>>,
        custom_tool_manager: Arc<Mutex<CustomToolManager>>,
    ) -> Self {
        let permission_manager = Arc::new(PermissionManager::new(config.clone()));
        let audit_logger = Arc::new(AuditLogger::new());
        let usage_stats = Arc::new(Mutex::new(UsageStats::new()));
        let pinned_files = Arc::new(Mutex::new(Vec::new()));

        // Create ToolExecutor with all the managers
        let tool_executor = Arc::new(ToolExecutor::new(
            mcp_manager.clone(),
            lsp_manager.clone(),
            skill_manager.clone(),
            custom_tool_manager.clone(),
            permission_manager.clone(),
            Arc::new(WebManager::new()),
            Arc::new(Mutex::new(Planner::new())),
            Arc::new(Mutex::new(RagManager::new(&config))),
        ));

        Self {
            client: Client::new(),
            config,
            mcp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
            audit_logger,
            usage_stats,
            pinned_files,
            tool_executor,
        }
    }

    #[async_recursion]
    pub async fn send_message(
        &self,
        messages_history: &mut Vec<Value>,
        prompt: &str,
        progress_tx: mpsc::Sender<String>,
        approval_rx: &mut mpsc::Receiver<bool>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        if messages_history.is_empty() {
            let rules_content = rules::load_rules(&self.config.instructions);
            let mut system_prompt = format!(
                "{}\n\n## Instructions and Rules\n{}",
                BASE_SYSTEM_PROMPT, rules_content
            );

            // Skills integration
            {
                let skills = self.skill_manager.lock().await;
                let available_skills = skills.get_available_skills_xml();
                system_prompt.push_str("\n\n## Available Skills\n");
                system_prompt.push_str(&available_skills);
            }

            // Pinned files integration
            {
                let pinned = self.pinned_files.lock().await;
                if !pinned.is_empty() {
                    system_prompt.push_str("\n\n## Pinned Context\n");
                    for path in pinned.iter() {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            system_prompt.push_str(&format!(
                                "<file path=\"{}\">\n{}\n</file>\n",
                                path, content
                            ));
                        }
                    }
                }
            }

            messages_history.push(json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        messages_history.push(json!({
            "role": "user",
            "content": prompt
        }));

        // Auto-Context Summarization: summarize old messages when approaching budget
        let (summarized, summary) = self.check_and_summarize_context(messages_history).await;
        if summarized {
            if let Some(s) = summary {
                let _ = progress_tx
                    .send(format!("open_crust: Summarized old context: {}", s))
                    .await;
            }
        } else {
            // Fall back to basic pruning if summarization didn't trigger
            if messages_history.len() > 22 {
                let _ = progress_tx
                    .send(String::from(
                        "open_crust: Pruning old context to stay within limits...",
                    ))
                    .await;
                let system_prompt = messages_history.remove(0);
                let last_messages = messages_history.split_off(messages_history.len() - 20);
                *messages_history = vec![system_prompt];
                messages_history.extend(last_messages);
            }
        }

        loop {
            let res = match self.config.provider {
                ProviderType::Ollama => self.generate_ollama(messages_history).await?,
                ProviderType::OpenRouter => self.generate_openrouter(messages_history).await?,
                ProviderType::OpenAI => self.generate_openai(messages_history).await?,
                ProviderType::Gemini => self.generate_gemini(messages_history).await?,
                ProviderType::Mistral => self.generate_mistral(messages_history).await?,
                ProviderType::Anthropic => self.generate_anthropic(messages_history).await?,
            };

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

                    // Get input summary for permission check and audit
                    let input_summary = match name {
                        "bash" => args.get("command").and_then(|v| v.as_str()).unwrap_or(""),
                        "write" => args.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        "read" => args.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        _ => args_str,
                    };

                    // Check if tool requires approval
                    let permission = self
                        .permission_manager
                        .check_permission(name, input_summary);
                    let approved = match permission {
                        PermissionAction::Allow => true,
                        PermissionAction::Deny => false,
                        PermissionAction::Ask => {
                            // Handle tools that need user approval
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
                                let _ = progress_tx.send(format!("open_crust: [APPROVAL_REQUIRED] The agent wants to run '{}' with input: '{}'. Allow? (y/n)", name, input_summary)).await;
                            }
                            approval_rx.recv().await.unwrap_or(false)
                        }
                    };

                    self.audit_logger.log_action(name, input_summary, approved);

                    let result = if approved {
                        let _ = progress_tx
                            .send(format!("open_crust: Executing tool '{}'...", name))
                            .await;

                        // Use ToolExecutor to execute the tool
                        match self.tool_executor.execute(name, &args).await {
                            Ok(res) => res,
                            Err(e) => format!("Error executing tool '{}': {}", name, e),
                        }
                    } else {
                        let _ = progress_tx
                            .send(format!("open_crust: Tool '{}' denied by user.", name))
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
                return Ok(content.to_string());
            } else {
                // If it neither has content nor tool_calls (rare, but can happen), just stop.
                return Err("Empty response".into());
            }
        }
    }

    async fn generate_completion(
        &self,
        messages: &[Value],
        url: &str,
        auth_header: Option<(&str, String)>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Assemble tool schemas using ToolExecutor
        let tools_schema = crate::tool_executor::get_all_tool_schemas(
            &self.mcp_manager,
            &self.custom_tool_manager,
        )
        .await;

        // Build request body
        let body = json!({
            "model": self.config.model,
            "messages": messages,
            "tools": tools_schema,
            "stream": false
        });

        // Make HTTP request with optional auth header
        let mut request = self.client.post(url).json(&body);
        if let Some((header_name, header_value)) = auth_header {
            request = request.header(header_name, header_value);
        }
        let res = request.send().await?;
        let res_json: Value = res.json().await?;

        // Extract and track usage stats
        if let Some(usage) = res_json.get("usage") {
            let input = usage
                .get("prompt_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);
            let output = usage
                .get("completion_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);
            let mut stats = self.usage_stats.lock().await;
            stats.add_usage(&self.config.model, input, output);
        }

        Ok(res_json)
    }

    async fn generate_ollama(
        &self,
        messages: &[Value],
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let ollama_url = self
            .config
            .ollama_url
            .as_deref()
            .unwrap_or("http://localhost:11434");
        // Use Ollama-native endpoint
        self.generate_completion(messages, &format!("{}/api/chat", ollama_url), None)
            .await
    }

    async fn generate_openrouter(
        &self,
        messages: &[Value],
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.openrouter_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://openrouter.ai/api/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    async fn generate_openai(
        &self,
        messages: &[Value],
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.openai_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.openai.com/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    async fn generate_gemini(
        &self,
        messages: &[Value],
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.gemini_api_key.as_deref().unwrap_or("");
        // Use Google's OpenAI-compatible endpoint for easier integration
        let res_json = self
            .generate_completion(
                messages,
                "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    async fn generate_mistral(
        &self,
        messages: &[Value],
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.mistral_api_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.mistral.ai/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    async fn generate_anthropic(
        &self,
        messages: &[Value],
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.anthropic_api_key.as_deref().unwrap_or("");

        // Convert messages to Anthropic format (skip system messages)
        let mut anthropic_messages: Vec<Value> = Vec::new();
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str());
            let content = msg.get("content").and_then(|c| c.as_str());
            if let (Some(r), Some(c)) = (role, content)
                && r != "system"
            {
                anthropic_messages.push(json!({"role": r, "content": c}));
            }
        }

        let body = json!({
            "model": self.config.model,
            "messages": anthropic_messages,
            "max_tokens": 4096
        });

        let client = reqwest::Client::new();
        let res = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let res_json: Value = res.json().await?;
        Ok(res_json
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .cloned()
            .unwrap_or(json!({})))
    }

    /// Simple query without tool execution - for multi-agent comparison
    pub async fn query_simple(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let messages = vec![json!({"role": "user", "content": prompt})];

        let res = match self.config.provider {
            ProviderType::Ollama => self.generate_ollama(&messages).await?,
            ProviderType::OpenRouter => self.generate_openrouter(&messages).await?,
            ProviderType::OpenAI => self.generate_openai(&messages).await?,
            ProviderType::Gemini => self.generate_gemini(&messages).await?,
            ProviderType::Mistral => self.generate_mistral(&messages).await?,
            ProviderType::Anthropic => self.generate_anthropic(&messages).await?,
        };

        // Extract content from response - handle multiple formats:
        // 1. Direct "content" field (some APIs)
        // 2. OpenAI-compatible: choices[0].message.content
        // 3. Ollama-native: message.content
        if let Some(content) = res.get("content").and_then(|c| c.as_str()) {
            Ok(content.to_string())
        } else if let Some(choices) = res.get("choices").and_then(|c| c.as_array()) {
            if let Some(first_choice) = choices.first()
                && let Some(message) = first_choice.get("message")
                && let Some(content) = message.get("content").and_then(|c| c.as_str())
            {
                return Ok(content.to_string());
            }
            Err("No content in response".into())
        } else if let Some(message) = res.get("message") {
            // Ollama-native format
            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                return Ok(content.to_string());
            }
            Err("No content in message".into())
        } else {
            Err("No content in response".into())
        }
    }

    /// Auto-summarize context when approaching budget (80% threshold)
    /// Returns (should_summarize, summary_message)
    pub async fn check_and_summarize_context(
        &self,
        messages_history: &mut Vec<Value>,
    ) -> (bool, Option<String>) {
        // Calculate current token usage (rough estimate: 4 chars per token)
        let total_chars: usize = messages_history
            .iter()
            .map(|m| {
                m.get("content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0)
            })
            .sum();
        let estimated_tokens = (total_chars / 4) as u64;

        let context_limit = self.config.context_limit();
        let threshold = (context_limit as f64 * self.config.summarization_threshold()) as u64;

        if estimated_tokens < threshold {
            return (false, None);
        }

        // Find system prompt (first message)
        let system_prompt = if let Some(first) = messages_history.first() {
            if first.get("role").and_then(|r| r.as_str()) == Some("system") {
                Some(first.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Collect old messages to summarize (all except system prompt and last 10)
        if messages_history.len() <= 11 {
            return (false, None); // Not enough messages to summarize
        }

        let split_point = messages_history.len() - 10;
        let old_messages: Vec<Value> = messages_history.drain(..split_point).collect();

        // Build summarization prompt
        let messages_to_summarize: Vec<String> = old_messages
            .iter()
            .filter_map(|m| {
                let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
                let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if !content.is_empty() {
                    Some(format!("{}: {}", role, content))
                } else {
                    None
                }
            })
            .collect();

        let summarize_prompt = format!(
            "Please provide a concise summary of the following conversation history, preserving key technical details, decisions, and context:\n\n{}",
            messages_to_summarize.join("\n")
        );

        // Generate summary using a simple query
        // For now, we'll use the existing generate methods
        // In production, this would use a faster/cheaper model
        let summary_result = match self.config.provider {
            crate::config::ProviderType::Ollama => {
                let messages = [json!({"role": "user", "content": &summarize_prompt})];
                self.generate_ollama(&messages).await
            }
            crate::config::ProviderType::OpenRouter => {
                let messages = [json!({"role": "user", "content": &summarize_prompt})];
                self.generate_openrouter(&messages).await
            }
            crate::config::ProviderType::OpenAI => {
                let messages = [json!({"role": "user", "content": &summarize_prompt})];
                self.generate_openai(&messages).await
            }
            crate::config::ProviderType::Gemini => {
                let messages = [json!({"role": "user", "content": &summarize_prompt})];
                self.generate_gemini(&messages).await
            }
            crate::config::ProviderType::Mistral => {
                let messages = [json!({"role": "user", "content": &summarize_prompt})];
                self.generate_mistral(&messages).await
            }
            crate::config::ProviderType::Anthropic => {
                let messages = [json!({"role": "user", "content": &summarize_prompt})];
                self.generate_anthropic(&messages).await
            }
        };

        let summary = match summary_result {
            Ok(mut res) => res
                .get_mut("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            Err(_) => "Previous conversation context (summarized due to length)".to_string(),
        };

        // Rebuild message history: system prompt + summary + recent messages
        let mut new_history = Vec::new();
        if let Some(sp) = system_prompt {
            new_history.push(sp);
        }
        new_history.push(json!({
            "role": "system",
            "content": format!("[Previous conversation summary: {}]", summary)
        }));
        // messages_history now only contains the last 10 messages (after drain)
        new_history.append(messages_history);

        *messages_history = new_history;

        (true, Some(summary))
    }

    /// Generate lightweight input completion (ghost text)
    /// Takes current input and returns a short suggestion
    pub async fn generate_input_completion(
        &self,
        current_input: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if current_input.trim().is_empty() {
            return Ok(String::new());
        }

        // Truncate input to avoid large context
        let truncated = if current_input.len() > 200 {
            &current_input[current_input.len() - 200..]
        } else {
            current_input
        };

        let prompt = format!(
            "Complete the following input with a short, relevant continuation (max 50 chars). Only return the continuation, no quotes, no explanation:\n\n{}",
            truncated
        );

        let messages = vec![json!({"role": "user", "content": &prompt})];

        let result = match self.config.provider {
            crate::config::ProviderType::Ollama => self.generate_ollama(&messages).await?,
            crate::config::ProviderType::OpenRouter => self.generate_openrouter(&messages).await?,
            crate::config::ProviderType::OpenAI => self.generate_openai(&messages).await?,
            crate::config::ProviderType::Gemini => self.generate_gemini(&messages).await?,
            crate::config::ProviderType::Mistral => self.generate_mistral(&messages).await?,
            crate::config::ProviderType::Anthropic => self.generate_anthropic(&messages).await?,
        };

        // Extract content from the response based on provider
        let content = match self.config.provider {
            crate::config::ProviderType::Ollama => result
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or(""),
            crate::config::ProviderType::OpenRouter
            | crate::config::ProviderType::OpenAI
            | crate::config::ProviderType::Mistral => result
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or(""),
            crate::config::ProviderType::Gemini => result
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or(""),
            crate::config::ProviderType::Anthropic => result
                .get("content")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or(""),
        };
        Ok(content.to_string())
    }
}
