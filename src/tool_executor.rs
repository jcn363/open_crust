//! Tool execution module for OpenCrust
//!
//! This module handles the execution of all tools (built-in, MCP, custom).
//! It provides a struct-based abstraction for tool execution and integrates
//! with the security module for safe operations.

use crate::mcp::McpManager;
use crate::lsp::LspManager;
use crate::skills::SkillManager;
use crate::permissions::PermissionManager;
use crate::custom_tools::CustomToolManager;
use crate::web::WebManager;
use crate::planner::Planner;
use crate::rag::RagManager;
use crate::security::{validate_path, validate_command};
use crate::tools;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Result type for tool execution
pub type ToolResult = Result<String, Box<dyn Error + Send + Sync>>;

/// Main tool executor that coordinates all tool types
pub struct ToolExecutor {
    pub mcp_manager: Arc<Mutex<McpManager>>,
    pub lsp_manager: Arc<Mutex<LspManager>>,
    pub skill_manager: Arc<Mutex<SkillManager>>,
    pub custom_tool_manager: Arc<Mutex<CustomToolManager>>,
    pub permission_manager: Arc<PermissionManager>,
    pub web_manager: Arc<WebManager>,
    pub planner: Arc<Mutex<Planner>>,
    pub rag_manager: Arc<RagManager>,
}

impl ToolExecutor {
    pub fn new(
        mcp_manager: Arc<Mutex<McpManager>>,
        lsp_manager: Arc<Mutex<LspManager>>,
        skill_manager: Arc<Mutex<SkillManager>>,
        custom_tool_manager: Arc<Mutex<CustomToolManager>>,
        permission_manager: Arc<PermissionManager>,
        web_manager: Arc<WebManager>,
        planner: Arc<Mutex<Planner>>,
        rag_manager: Arc<RagManager>,
    ) -> Self {
        Self {
            mcp_manager,
            lsp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
            web_manager,
            planner,
            rag_manager,
        }
    }

    /// Execute a tool by name
    pub async fn execute(&self, name: &str, args: &Value) -> ToolResult {
        match name {
            // Built-in tools
            "bash" => self.execute_bash(args).await,
            "read" => self.execute_read(args).await,
            "write" => self.execute_write(args).await,
            
            // LSP tools
            "lsp_goto_definition" => self.execute_lsp_goto_definition(args).await,
            "lsp_hover" => self.execute_lsp_hover(args).await,
            "lsp_find_references" => self.execute_lsp_find_references(args).await,
            "lsp_type_definition" => self.execute_lsp_type_definition(args).await,
            
            // Task and web tools
            "task" => self.execute_task(args).await,
            "web_search" => self.execute_web_search(args).await,
            "fetch_url" => self.execute_fetch_url(args).await,
            
            // Pin management
            "pin" => self.execute_pin(args).await,
            "unpin" => self.execute_unpin(args).await,
            
            // Planning tools
            "create_plan" => self.execute_create_plan(args).await,
            "mark_step_complete" => self.execute_mark_step_complete(args).await,
            
            // Search tools
            "semantic_search" => self.execute_semantic_search(args),
            "global_search_replace" => self.execute_global_search_replace(args).await,
            
            // Skill tool
            "skill" => self.execute_skill(args).await,
            
            // Try custom tools
            _ => {
                let custom_result = {
                    let custom = self.custom_tool_manager.lock().await;
                    custom.call_tool(name, args).await
                };
                
                match custom_result {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        // Try MCP tools
                        let mut mcp = self.mcp_manager.lock().await;
                        match mcp.call_tool(name, args).await {
                            Ok(result) => Ok(result),
                            Err(_) => {
                                // Fallback to built-in tools
                                Ok(tools::execute_tool(name, args))
                            }
                        }
                    }
                }
            }
        }
    }

    async fn execute_bash(&self, args: &Value) -> ToolResult {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Validate command for safety
        validate_command(command)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        
        match Command::new("sh").arg("-c").arg(command).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr))
            }
            Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        }
    }

    async fn execute_read(&self, args: &Value) -> ToolResult {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Validate path
        let validated_path = validate_path(path)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        
        match fs::read_to_string(&validated_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        }
    }

    async fn execute_write(&self, args: &Value) -> ToolResult {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Validate path
        let validated_path = validate_path(path_str)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        
        match fs::write(&validated_path, content) {
            Ok(_) => {
                // Format the file if possible
                crate::formatters::format_file(std::path::Path::new(&validated_path));
                Ok(format!("Successfully wrote to {}", validated_path.display()))
            }
            Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        }
    }

    async fn execute_lsp_goto_definition(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        
        let mut lsp = self.lsp_manager.lock().await;
        match lsp.goto_definition(path, line, character).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("LSP Error: {}", e)),
        }
    }

    async fn execute_lsp_hover(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        
        let mut lsp = self.lsp_manager.lock().await;
        match lsp.hover(path, line, character).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("LSP Error: {}", e)),
        }
    }

    async fn execute_lsp_find_references(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        
        let mut lsp = self.lsp_manager.lock().await;
        match lsp.find_references(path, line, character).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("LSP Error: {}", e)),
        }
    }

    async fn execute_lsp_type_definition(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        
        let mut lsp = self.lsp_manager.lock().await;
        match lsp.type_definition(path, line, character).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("LSP Error: {}", e)),
        }
    }

    async fn execute_task(&self, args: &Value) -> ToolResult {
        let sub_prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
        
        // This would need access to the LLM client - simplified for now
        Ok(format!("Subtask '{}' would be executed here", sub_prompt))
    }

    async fn execute_web_search(&self, args: &Value) -> ToolResult {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        match self.web_manager.search(query).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("Web Search Error: {}", e)),
        }
    }

    async fn execute_fetch_url(&self, args: &Value) -> ToolResult {
        let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
        
        // Check network permissions
        if !self.permission_manager.check_network_permission(url) {
            return Ok(format!("Permission Denied: Domain not whitelisted for URL '{}'", url));
        }
        
        match self.web_manager.fetch_url(url).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("Fetch Error: {}", e)),
        }
    }

    async fn execute_pin(&self, args: &Value) -> ToolResult {
        // Pin functionality is handled by LlmClient's pinned_files
        // This is a placeholder - actual implementation would need access to pinned_files
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        Ok(format!("Pin requested for: {}", path))
    }

    async fn execute_unpin(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        Ok(format!("Unpin requested for: {}", path))
    }

    async fn execute_create_plan(&self, args: &Value) -> ToolResult {
        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled Plan");
        let steps = args.get("steps")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        
        let mut planner = self.planner.lock().await;
        Ok(planner.create_plan(title, steps))
    }

    async fn execute_mark_step_complete(&self, args: &Value) -> ToolResult {
        let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let mut planner = self.planner.lock().await;
        Ok(planner.mark_step_complete(index))
    }

    fn execute_semantic_search(&self, args: &Value) -> ToolResult {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        Ok(self.rag_manager.semantic_search(query))
    }

    async fn execute_global_search_replace(&self, args: &Value) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
        let replacement = args.get("replacement").and_then(|v| v.as_str()).unwrap_or("");
        let include = args.get("include").and_then(|v| v.as_str()).unwrap_or("*");
        
        // Use Rust-native implementation instead of shell commands
        self.search_replace_rust(pattern, replacement, include).await
    }

    async fn search_replace_rust(&self, pattern: &str, replacement: &str, include: &str) -> ToolResult {
        use std::fs;
        use walkdir::WalkDir;
        
        // Simple implementation - in production, use a proper regex library
        let mut count = 0;
        for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if !include.contains(&ext.to_string_lossy().to_string()) {
                    continue;
                }
            }
            
            if let Ok(content) = fs::read_to_string(path) {
                if content.contains(pattern) {
                    let new_content = content.replace(pattern, replacement);
                    if new_content != content {
                        fs::write(path, new_content)?;
                        count += 1;
                    }
                }
            }
        }
        
        Ok(format!("Replaced '{}' with '{}' in {} files", pattern, replacement, count))
    }

    async fn execute_skill(&self, args: &Value) -> ToolResult {
        let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let skills = self.skill_manager.lock().await;
        match skills.skills.get(name) {
            Some(skill) => Ok(skill.content.clone()),
            None => Ok(format!("Skill '{}' not found.", name)),
        }
    }
}

/// Get combined tool schemas from all sources
pub async fn get_all_tool_schemas(
    mcp_manager: &Arc<Mutex<McpManager>>,
    custom_tool_manager: &Arc<Mutex<CustomToolManager>>,
) -> Value {
    let mut tools_schema = tools::get_tools_schema();
    
    if let Some(tools_array) = tools_schema.as_array_mut() {
        // Add MCP tools
        let mut mcp = mcp_manager.lock().await;
        let mcp_tools = mcp.list_tools().await;
        tools_array.extend(mcp_tools);
        
        // Add custom tools
        let custom = custom_tool_manager.lock().await;
        tools_array.extend(custom.get_tools_schema());
    }
    
    tools_schema
}
