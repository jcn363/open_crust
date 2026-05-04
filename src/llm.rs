use crate::config::{Config, ProviderType, PermissionAction};
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use tokio::sync::mpsc;
use crate::tools;
use crate::rules;

const BASE_SYSTEM_PROMPT: &str = "You are open_crust, a pure Rust terminal-based AI coding agent. 
You have access to tools to interact with the local filesystem and execute bash commands.
Always follow the project's rules and guidelines provided below.";

use crate::mcp::McpManager;
use crate::lsp::LspManager;
use crate::skills::SkillManager;
use crate::permissions::PermissionManager;
use crate::custom_tools::CustomToolManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_recursion::async_recursion;

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    pub config: Config,
    pub mcp_manager: Arc<Mutex<McpManager>>,
    pub lsp_manager: Arc<Mutex<LspManager>>,
    pub skill_manager: Arc<Mutex<SkillManager>>,
    pub custom_tool_manager: Arc<Mutex<CustomToolManager>>,
    pub permission_manager: Arc<PermissionManager>,
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
        Self {
            client: Client::new(),
            config,
            mcp_manager,
            lsp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
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
            let mut system_prompt = format!("{}\n\n## Instructions and Rules\n{}", BASE_SYSTEM_PROMPT, rules_content);
            
            // Skills integration
            {
                let skills = self.skill_manager.lock().await;
                let available_skills = skills.get_available_skills_xml();
                system_prompt.push_str("\n\n## Available Skills\n");
                system_prompt.push_str(&available_skills);
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

        loop {
            let res = match self.config.provider {
                ProviderType::Ollama => self.generate_ollama(messages_history).await?,
                ProviderType::OpenRouter => self.generate_openrouter(messages_history).await?,
            };

            if let Some(tool_calls) = res.get("tool_calls").and_then(|t| t.as_array()) {
                messages_history.push(res.clone());
                
                for tool_call in tool_calls {
                    let id = tool_call.get("id").and_then(|i| i.as_str()).unwrap_or("");
                    let name = tool_call.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                    let args_str = tool_call.get("function").and_then(|f| f.get("arguments")).and_then(|a| a.as_str()).unwrap_or("{}");
                    let args: Value = serde_json::from_str(args_str).unwrap_or(json!({}));
                    
                    let input_summary = match name {
                        "bash" => args.get("command").and_then(|v| v.as_str()).unwrap_or(""),
                        "write" => args.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        "read" => args.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        "edit" => args.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        _ => args_str,
                    };

                    let permission = self.permission_manager.check_permission(name, input_summary);
                    let approved = match permission {
                        PermissionAction::Allow => true,
                        PermissionAction::Deny => false,
                        PermissionAction::Ask => {
                            let _ = progress_tx.send(format!("open_crust: [APPROVAL_REQUIRED] The agent wants to run '{}' with input: '{}'. Allow? (y/n)", name, input_summary)).await;
                            approval_rx.recv().await.unwrap_or(false)
                        }
                    };

                    let result = if approved {
                        let _ = progress_tx.send(format!("open_crust: Executing tool '{}'...", name)).await;
                        
                        match name {
                            "lsp_goto_definition" => {
                                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let mut lsp = self.lsp_manager.lock().await;
                                match lsp.goto_definition(path, line, character).await {
                                    Ok(res) => res,
                                    Err(e) => format!("LSP Error: {}", e),
                                }
                            }
                            "lsp_hover" => {
                                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let mut lsp = self.lsp_manager.lock().await;
                                match lsp.hover(path, line, character).await {
                                    Ok(res) => res,
                                    Err(e) => format!("LSP Error: {}", e),
                                }
                            }
                            "lsp_find_references" => {
                                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let mut lsp = self.lsp_manager.lock().await;
                                match lsp.find_references(path, line, character).await {
                                    Ok(res) => res,
                                    Err(e) => format!("LSP Error: {}", e),
                                }
                            }
                            "lsp_type_definition" => {
                                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let mut lsp = self.lsp_manager.lock().await;
                                match lsp.type_definition(path, line, character).await {
                                    Ok(res) => res,
                                    Err(e) => format!("LSP Error: {}", e),
                                }
                            }
                            "task" => {
                                let sub_prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
                                let (p_tx, _) = mpsc::channel(100);
                                let (_, mut a_rx) = mpsc::channel(1);
                                let mut history = Vec::new();
                                match self.send_message(&mut history, sub_prompt, p_tx, &mut a_rx).await {
                                    Ok(res) => format!("Subtask completed: {}", res),
                                    Err(e) => format!("Subtask failed: {}", e),
                                }
                            }
                            "skill" => {
                                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                let skills = self.skill_manager.lock().await;
                                match skills.skills.get(name) {
                                    Some(skill) => skill.content.clone(),
                                    None => format!("Skill '{}' not found.", name),
                                }
                            }
                            _ => {
                                // Try Custom tool first
                                let custom = self.custom_tool_manager.lock().await;
                                match custom.call_tool(name, &args).await {
                                    Ok(res) => res,
                                    Err(_) => {
                                        // Try MCP tool
                                        let mut mcp = self.mcp_manager.lock().await;
                                        match mcp.call_tool(name, &args).await {
                                            Ok(res) => res,
                                            Err(_) => {
                                                // Fallback to built-in tools
                                                tools::execute_tool(name, &args)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        let _ = progress_tx.send(format!("open_crust: Tool '{}' denied by user.", name)).await;
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

    async fn generate_ollama(&self, messages: &[Value]) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let ollama_url = self.config.ollama_url.as_deref().unwrap_or("http://localhost:11434");
        let mut tools_schema = tools::get_tools_schema();
        let mut mcp = self.mcp_manager.lock().await;
        let mcp_tools = mcp.list_tools().await;
        if let Some(tools_array) = tools_schema.as_array_mut() {
            tools_array.extend(mcp_tools);
            let custom = self.custom_tool_manager.lock().await;
            tools_array.extend(custom.get_tools_schema());
        }

        let body = json!({
            "model": self.config.model,
            "messages": messages,
            "stream": false,
            "tools": tools_schema
        });

        let res = self.client.post(format!("{}/api/chat", ollama_url))
            .json(&body)
            .send()
            .await?;

        let res_json: Value = res.json().await?;
        Ok(res_json.get("message").cloned().unwrap_or(json!({})))
    }

    async fn generate_openrouter(&self, messages: &[Value]) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.openrouter_key.as_deref().unwrap_or("");
        
        let mut tools_schema = tools::get_tools_schema();
        let mut mcp = self.mcp_manager.lock().await;
        let mcp_tools = mcp.list_tools().await;
        if let Some(tools_array) = tools_schema.as_array_mut() {
            tools_array.extend(mcp_tools);
            let custom = self.custom_tool_manager.lock().await;
            tools_array.extend(custom.get_tools_schema());
        }

        let body = json!({
            "model": self.config.model,
            "messages": messages,
            "tools": tools_schema
        });

        let res = self.client.post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await?;

        let res_json: Value = res.json().await?;
        Ok(res_json.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }
}
