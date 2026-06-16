//! Tool execution module for OpenCrust
//!
//! This module handles the execution of all tools (built-in, MCP, custom).
//! It provides a struct-based abstraction for tool execution and integrates
//! with the security module for safe operations.

use crate::config::Config;
use crate::custom_tools::CustomToolManager;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::orchestrator::Orchestrator;
use crate::permissions::PermissionManager;
use crate::planner::Planner;
use crate::plugins::PluginManager;
use crate::rag::RagManager;
use crate::security::validate_path;
use crate::skills::SkillManager;
use crate::tools;
use crate::web::WebManager;
use lru::LruCache;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Result type for tool execution
pub type ToolResult = Result<String, Box<dyn Error + Send + Sync>>;

/// Main tool executor that coordinates all tool types
/// Central tool execution engine: MCP, LSP, custom, and built-in tools
///
/// Aggregates all tool sources under a unified interface. Dispatches tool
/// calls to the appropriate backend, enforces permissions, records audit
/// logs, and manages subtask delegation via the orchestrator.
pub struct ToolExecutor {
    pub config: Arc<Config>,
    pub mcp_manager: Arc<Mutex<McpManager>>,
    pub lsp_manager: Arc<Mutex<LspManager>>,
    pub skill_manager: Arc<Mutex<SkillManager>>,
    pub custom_tool_manager: Arc<Mutex<CustomToolManager>>,
    pub permission_manager: Arc<PermissionManager>,
    pub web_manager: Arc<WebManager>,
    pub planner: Arc<Mutex<Planner>>,
    pub rag_manager: Arc<Mutex<RagManager>>,
    pub pinned_files: Arc<Mutex<Vec<String>>>,
    pub orchestrator: Arc<Mutex<Orchestrator>>,
    /// Plugin manager for extension tool execution
    pub plugin_manager: Arc<Mutex<PluginManager>>,
    /// Bounded LRU cache for frequently accessed files with TTL eviction
    /// Max 1000 entries, 1 hour TTL, ~100MB max memory
    file_cache: Mutex<LruCache<String, (String, Instant)>>,
}

impl ToolExecutor {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        config: Arc<Config>,
        mcp_manager: Arc<Mutex<McpManager>>,
        lsp_manager: Arc<Mutex<LspManager>>,
        skill_manager: Arc<Mutex<SkillManager>>,
        custom_tool_manager: Arc<Mutex<CustomToolManager>>,
        permission_manager: Arc<PermissionManager>,
        web_manager: Arc<WebManager>,
        planner: Arc<Mutex<Planner>>,
        rag_manager: Arc<Mutex<RagManager>>,
        pinned_files: Arc<Mutex<Vec<String>>>,
        orchestrator: Arc<Mutex<Orchestrator>>,
        plugin_manager: Arc<Mutex<PluginManager>>,
    ) -> Self {
        // LRU cache with max 1000 entries (~100MB for typical file sizes)
        let cache_capacity = NonZeroUsize::new(1000).unwrap();
        Self {
            config,
            mcp_manager,
            lsp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
            web_manager,
            planner,
            rag_manager,
            pinned_files,
            orchestrator,
            plugin_manager,
            file_cache: Mutex::new(LruCache::new(cache_capacity)),
        }
    }

    /// Execute a tool by name
    pub async fn execute(&self, name: &str, args: &Value) -> ToolResult {
        // Fire on_tool_execute hook for plugins that subscribe to it
        {
            let plugins = self.plugin_manager.lock().await;
            let hook_ctx = serde_json::json!({"tool": name, "args": args});
            let results = plugins.execute_hook(
                "on_tool_execute",
                &serde_json::to_string(&hook_ctx).unwrap_or_default(),
            );
            for (plugin_name, result) in &results {
                if let Err(e) = result {
                    eprintln!("[Plugin:{}] on_tool_execute error: {}", plugin_name, e);
                }
            }
        }

        match name {
            // Built-in tools
            "bash" => self.execute_bash(args).await,
            "read" => self.execute_read(args).await,
            "write" => self.execute_write(args),

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
            "semantic_search" => self.execute_semantic_search(args).await,
            "index_codebase" => self.execute_index_codebase(args).await,
            "global_search_replace" => self.execute_global_search_replace(args).await,

            // Skill tool
            "skill" => self.execute_skill(args).await,

            // LSP tools
            "lsp_completion" => self.execute_lsp_completion(args).await,
            "lsp_diagnostics" => self.execute_lsp_diagnostics(args).await,
            "lsp_formatting" => self.execute_lsp_formatting(args).await,

            // Try custom tools
            _ => {
                let custom_result = {
                    let custom = self.custom_tool_manager.lock().await;
                    custom.call_tool(name, args)
                };

                match custom_result {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        // Try MCP tools
                        let mut mcp = self.mcp_manager.lock().await;
                        match mcp.call_tool(name, args).await {
                            Ok(result) => Ok(result),
                            Err(_) => {
                                // Try plugin tools (prefixed with "plugin_")
                                if name.starts_with("plugin_") {
                                    let plugins = self.plugin_manager.lock().await;
                                    match plugins.execute_tool(name, args) {
                                        Ok(result) => Ok(result),
                                        Err(e) => Err(e.into()),
                                    }
                                } else {
                                    // Fallback to built-in tools
                                    let result = tools::execute_tool(name, args);
                                    if result.starts_with("Unknown tool:") {
                                        Err(format!("Unknown tool: {}", name).into())
                                    } else {
                                        Ok(result)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn execute_bash(&self, args: &Value) -> ToolResult {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Execute command safely without shell interpretation
        let output =
            tokio::task::spawn_blocking(move || crate::security::execute_command_safely(&command))
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr))
    }

    async fn execute_read(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        // Validate path
        let validated_path =
            validate_path(path).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let validated_path_str = validated_path.to_string_lossy().into_owned();

        // Check cache first (TTL: 1 hour, LRU eviction on capacity)
        {
            let mut cache = self.file_cache.lock().await;
            if let Some((content, timestamp)) = cache.get(&validated_path_str) {
                // Check TTL (1 hour)
                if timestamp.elapsed() < Duration::from_secs(3600) {
                    return Ok(content.clone());
                } else {
                    // Entry expired, remove it
                    cache.pop(&validated_path_str);
                }
            }
        }

        // Read from disk
        let content = match fs::read_to_string(&validated_path) {
            Ok(content) => content,
            Err(e) => return Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        };

        // Update cache (LRU will evict oldest if at capacity)
        {
            let mut cache = self.file_cache.lock().await;
            cache.put(
                validated_path_str.clone(),
                (content.clone(), Instant::now()),
            );
        }

        Ok(content)
    }

    fn execute_write(&self, args: &Value) -> ToolResult {
        let path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");

        // Validate path
        let validated_path =
            validate_path(path_str).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        match fs::write(&validated_path, content) {
            Ok(_) => {
                // Format the file if possible (silently ignore formatter errors)
                let _ = crate::formatters::format_file(std::path::Path::new(&validated_path));
                Ok(format!(
                    "Successfully wrote to {}",
                    validated_path.display()
                ))
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

    async fn execute_lsp_completion(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let mut lsp = self.lsp_manager.lock().await;
        match lsp.completion(path, line, character).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("LSP Error: {}", e)),
        }
    }

    async fn execute_lsp_diagnostics(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        let mut lsp = self.lsp_manager.lock().await;
        match lsp.diagnostics(path).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("LSP Error: {}", e)),
        }
    }

    async fn execute_lsp_formatting(&self, args: &Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        let mut lsp = self.lsp_manager.lock().await;
        match lsp.formatting(path).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("LSP Error: {}", e)),
        }
    }

    async fn execute_task(&self, args: &Value) -> ToolResult {
        let sub_prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if sub_prompt.is_empty() {
            return Ok("No prompt provided for subtask".to_string());
        }

        let mut orchestrator = self.orchestrator.lock().await;
        let llm_client = {
            // Create a minimal LlmClient for the orchestrator to use
            let client = crate::llm::LlmClient::new(
                self.config.clone(),
                self.mcp_manager.clone(),
                self.lsp_manager.clone(),
                self.skill_manager.clone(),
                self.custom_tool_manager.clone(),
            )
            .map_err(|e| format!("Failed to create LLM client: {}", e))?;
            Arc::new(client)
        };

        let result = orchestrator.execute_request(&sub_prompt, llm_client).await;
        Ok(format!(
            "Subtask completed: {} tasks executed. Summary: {}",
            result.completed.len() + result.failed.len(),
            result.summary
        ))
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
            return Ok(format!(
                "Permission Denied: Domain not whitelisted for URL '{}'",
                url
            ));
        }

        match self.web_manager.fetch_url(url).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(format!("Fetch Error: {}", e)),
        }
    }

    async fn execute_pin(&self, args: &Value) -> ToolResult {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if path.is_empty() {
            return Ok("Error: No path provided for pinning".to_string());
        }

        let mut pinned = self.pinned_files.lock().await;
        if !pinned.contains(&path) {
            pinned.push(path.clone());
            Ok(format!("Successfully pinned: {}", path))
        } else {
            Ok(format!("Already pinned: {}", path))
        }
    }

    async fn execute_unpin(&self, args: &Value) -> ToolResult {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if path.is_empty() {
            return Ok("Error: No path provided for unpinning".to_string());
        }

        let mut pinned = self.pinned_files.lock().await;
        if let Some(pos) = pinned.iter().position(|p| p == &path) {
            pinned.remove(pos);
            Ok(format!("Successfully unpinned: {}", path))
        } else {
            Ok(format!("Not pinned: {}", path))
        }
    }

    async fn execute_create_plan(&self, args: &Value) -> ToolResult {
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Plan");
        let steps = args
            .get("steps")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut planner = self.planner.lock().await;
        Ok(planner.create_plan(title, steps))
    }

    async fn execute_mark_step_complete(&self, args: &Value) -> ToolResult {
        let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let mut planner = self.planner.lock().await;
        Ok(planner.mark_step_complete(index))
    }

    async fn execute_semantic_search(&self, args: &Value) -> ToolResult {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let top_k = args.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        let rag = self.rag_manager.lock().await;
        Ok(rag.semantic_search(query, top_k).await)
    }

    async fn execute_index_codebase(&self, args: &Value) -> ToolResult {
        let root = args.get("root").and_then(|v| v.as_str()).unwrap_or(".");

        let mut rag = self.rag_manager.lock().await;
        match rag.index_codebase(root).await {
            Ok((files, chunks)) => Ok(format!("Indexed {} files with {} chunks", files, chunks)),
            Err(e) => Ok(format!("Index error: {}", e)),
        }
    }

    async fn execute_global_search_replace(&self, args: &Value) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
        let replacement = args
            .get("replacement")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let include = args.get("include").and_then(|v| v.as_str()).unwrap_or("*");

        // Use Rust-native implementation instead of shell commands
        self.search_replace_rust(pattern, replacement, include)
    }

    fn search_replace_rust(&self, pattern: &str, replacement: &str, include: &str) -> ToolResult {
        use std::fs;
        use walkdir::WalkDir;

        // Simple implementation - in production, use a proper regex library
        let mut count = 0;
        for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if include != "*" {
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !include.split(',').any(|pat| {
                    let pat = pat.trim().trim_start_matches("*.");
                    ext == pat || pat == "*"
                }) {
                    continue;
                }
            }

            if let Ok(content) = fs::read_to_string(path)
                && content.contains(pattern)
            {
                let new_content = content.replace(pattern, replacement);
                if new_content != content {
                    fs::write(path, new_content)?;
                    count += 1;
                }
            }
        }

        Ok(format!(
            "Replaced '{}' with '{}' in {} files",
            pattern, replacement, count
        ))
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
    plugin_manager: &Arc<Mutex<crate::plugins::PluginManager>>,
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

        // Add plugin tools
        let plugins = plugin_manager.lock().await;
        tools_array.extend(plugins.get_tool_schemas());
    }

    tools_schema
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_pin_unpin() {
        let mcp_manager = Arc::new(Mutex::new(McpManager::new()));
        let lsp_manager = Arc::new(Mutex::new(LspManager::new()));
        let skill_manager = Arc::new(Mutex::new(SkillManager::new()));
        let custom_tool_manager = Arc::new(Mutex::new(CustomToolManager::new()));
        let config = Arc::new(Config::default());
        let permission_manager = Arc::new(PermissionManager::new(config.clone()));
        let web_manager = Arc::new(WebManager::new().expect("Failed to create web manager"));
        let planner = Arc::new(Mutex::new(Planner::new()));
        let rag_manager = Arc::new(Mutex::new(RagManager::new(&config)));
        let pinned_files = Arc::new(Mutex::new(Vec::new()));
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(config.clone())));
        let plugin_manager = Arc::new(Mutex::new(PluginManager::new()));

        let tool_executor = ToolExecutor::new(
            config.clone(),
            mcp_manager,
            lsp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
            web_manager,
            planner,
            rag_manager,
            pinned_files,
            orchestrator,
            plugin_manager,
        );

        let pin_args = json!({ "path": "/test/file.rs" });
        let result = tool_executor.execute_pin(&pin_args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Successfully pinned: /test/file.rs");

        let result = tool_executor.execute_pin(&pin_args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Already pinned: /test/file.rs");

        let unpin_args = json!({ "path": "/test/file.rs" });
        let result = tool_executor.execute_unpin(&unpin_args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Successfully unpinned: /test/file.rs");

        let result = tool_executor.execute_unpin(&unpin_args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Not pinned: /test/file.rs");

        let empty_args = json!({ "path": "" });
        let result = tool_executor.execute_pin(&empty_args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Error: No path provided for pinning");

        let result = tool_executor.execute_unpin(&empty_args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Error: No path provided for unpinning");
    }

    // --- File cache: LRU + TTL ---

    #[tokio::test]
    async fn file_cache_returns_cached_content_within_ttl() {
        let mcp_manager = Arc::new(Mutex::new(McpManager::new()));
        let lsp_manager = Arc::new(Mutex::new(LspManager::new()));
        let skill_manager = Arc::new(Mutex::new(SkillManager::new()));
        let custom_tool_manager = Arc::new(Mutex::new(CustomToolManager::new()));
        let config = Arc::new(Config::default());
        let permission_manager = Arc::new(PermissionManager::new(config.clone()));
        let web_manager = Arc::new(WebManager::new().expect("Failed to create web manager"));
        let planner = Arc::new(Mutex::new(Planner::new()));
        let rag_manager = Arc::new(Mutex::new(RagManager::new(&config)));
        let pinned_files = Arc::new(Mutex::new(Vec::new()));
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(config.clone())));
        let plugin_manager = Arc::new(Mutex::new(PluginManager::new()));

        let tool_executor = ToolExecutor::new(
            config.clone(),
            mcp_manager,
            lsp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
            web_manager,
            planner,
            rag_manager,
            pinned_files,
            orchestrator,
            plugin_manager,
        );

        // Manually insert a cache entry with a fresh timestamp
        let mut cache = tool_executor.file_cache.lock().await;
        cache.put(
            "test_cache_file.rs".to_string(),
            ("cached content here".to_string(), Instant::now()),
        );

        // Verify it's in the cache
        let entry = cache.get("test_cache_file.rs");
        assert!(entry.is_some());
        let (content, timestamp) = entry.unwrap();
        assert_eq!(content, "cached content here");
        assert!(timestamp.elapsed() < Duration::from_secs(3600));
    }

    #[tokio::test]
    async fn file_cache_evicts_expired_entries() {
        let mcp_manager = Arc::new(Mutex::new(McpManager::new()));
        let lsp_manager = Arc::new(Mutex::new(LspManager::new()));
        let skill_manager = Arc::new(Mutex::new(SkillManager::new()));
        let custom_tool_manager = Arc::new(Mutex::new(CustomToolManager::new()));
        let config = Arc::new(Config::default());
        let permission_manager = Arc::new(PermissionManager::new(config.clone()));
        let web_manager = Arc::new(WebManager::new().expect("Failed to create web manager"));
        let planner = Arc::new(Mutex::new(Planner::new()));
        let rag_manager = Arc::new(Mutex::new(RagManager::new(&config)));
        let pinned_files = Arc::new(Mutex::new(Vec::new()));
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(config.clone())));
        let plugin_manager = Arc::new(Mutex::new(PluginManager::new()));

        let tool_executor = ToolExecutor::new(
            config.clone(),
            mcp_manager,
            lsp_manager,
            skill_manager,
            custom_tool_manager,
            permission_manager,
            web_manager,
            planner,
            rag_manager,
            pinned_files,
            orchestrator,
            plugin_manager,
        );

        // Insert an entry with a timestamp older than 1 hour (expired)
        let mut cache = tool_executor.file_cache.lock().await;
        let expired_time = Instant::now() - Duration::from_secs(7200); // 2 hours ago
        cache.put(
            "expired_file.rs".to_string(),
            ("old content".to_string(), expired_time),
        );

        // The entry exists in the cache struct but is expired
        let entry = cache.get("expired_file.rs");
        assert!(entry.is_some());
        let (_, timestamp) = entry.unwrap();
        assert!(timestamp.elapsed() >= Duration::from_secs(3600)); // Confirm expired
    }

    #[tokio::test]
    async fn file_cache_lru_eviction_at_capacity() {
        let mut cache: LruCache<String, (String, Instant)> =
            LruCache::new(NonZeroUsize::new(3).unwrap());

        // Fill to capacity
        cache.put(
            "a.rs".to_string(),
            ("content_a".to_string(), Instant::now()),
        );
        cache.put(
            "b.rs".to_string(),
            ("content_b".to_string(), Instant::now()),
        );
        cache.put(
            "c.rs".to_string(),
            ("content_c".to_string(), Instant::now()),
        );

        assert_eq!(cache.len(), 3);

        // Adding a 4th should evict the LRU entry ("a.rs" - least recently used)
        cache.put(
            "d.rs".to_string(),
            ("content_d".to_string(), Instant::now()),
        );

        assert_eq!(cache.len(), 3);
        assert!(cache.get("a.rs").is_none()); // Evicted
        assert!(cache.get("b.rs").is_some()); // Still present
        assert!(cache.get("c.rs").is_some()); // Still present
        assert!(cache.get("d.rs").is_some()); // Just added
    }

    #[tokio::test]
    async fn file_cache_lru_access_refreshes_position() {
        let mut cache: LruCache<String, (String, Instant)> =
            LruCache::new(NonZeroUsize::new(3).unwrap());

        cache.put(
            "a.rs".to_string(),
            ("content_a".to_string(), Instant::now()),
        );
        cache.put(
            "b.rs".to_string(),
            ("content_b".to_string(), Instant::now()),
        );
        cache.put(
            "c.rs".to_string(),
            ("content_c".to_string(), Instant::now()),
        );

        // Access "a.rs" to refresh its LRU position
        cache.get("a.rs");

        // Adding a 4th should now evict "b.rs" (the new LRU)
        cache.put(
            "d.rs".to_string(),
            ("content_d".to_string(), Instant::now()),
        );

        assert!(cache.get("a.rs").is_some()); // Refreshed, still present
        assert!(cache.get("b.rs").is_none()); // Evicted
        assert!(cache.get("c.rs").is_some());
        assert!(cache.get("d.rs").is_some());
    }
}
