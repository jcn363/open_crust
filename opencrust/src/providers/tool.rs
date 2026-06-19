//! Tool provider trait for extensible tool execution
//!
//! Allows external plugins to provide custom tool implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::tool_executor::ToolResult;

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Working directory
    pub working_dir: String,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Current session ID
    pub session_id: Option<String>,
    /// User-provided metadata
    pub metadata: HashMap<String, Value>,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            working_dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            env: std::env::vars().collect(),
            session_id: None,
            metadata: HashMap::new(),
        }
    }
}

/// Tool invocation request
#[derive(Debug, Clone)]
pub struct ToolInvocation {
    /// Tool name
    pub name: String,
    /// Tool arguments as JSON
    pub arguments: Value,
    /// Execution context
    pub context: ToolContext,
}

/// Tool provider trait
#[async_trait]
pub trait ToolProvider: crate::providers::Provider {
    /// List of tool names this provider offers
    fn tools(&self) -> Vec<String>;

    /// Check if a specific tool is available
    fn has_tool(&self, name: &str) -> bool {
        self.tools().contains(&name.to_string())
    }

    /// Execute a tool
    async fn execute(&self, invocation: ToolInvocation) -> ToolResult;

    /// Get tool schema (for LLM function calling)
    fn schema(&self, name: &str) -> Option<Value>;
}

/// Registry for tool providers
pub type ToolProviderRegistry = crate::providers::ProviderRegistry<dyn ToolProvider>;

/// Create default tool provider registry
pub fn default_tool_registry() -> ToolProviderRegistry {
    let registry = ToolProviderRegistry::new();
    // Built-in tools are registered elsewhere
    registry
}
