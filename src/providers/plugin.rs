//! Plugin provider trait for external plugin integration
//!
//! Allows plugins to register as providers for various integration points.

use crate::plugins::types::Plugin;
use crate::providers::Provider;
use serde_json::Value;

/// Plugin provider trait - wraps a plugin as a provider
pub trait PluginProvider: Provider {
    /// Get the underlying plugin
    fn plugin(&self) -> &Plugin;

    /// Execute a hook
    fn execute_hook(&self, hook: &str, context: &str) -> Result<String, String>;

    /// Execute a tool
    fn execute_tool(&self, tool_name: &str, args: &Value) -> Result<String, String>;

    /// Get tool schemas
    fn tool_schemas(&self) -> Vec<Value>;
}

/// Wrapper that makes a Plugin implement PluginProvider
pub struct PluginWrapper {
    plugin: Plugin,
}

impl PluginWrapper {
    pub fn new(plugin: Plugin) -> Self {
        Self { plugin }
    }
}

impl Provider for PluginWrapper {
    fn id(&self) -> &str {
        &self.plugin.name
    }

    fn name(&self) -> &str {
        &self.plugin.name
    }

    fn is_available(&self) -> bool {
        self.plugin.enabled
    }

    fn priority(&self) -> u8 {
        40 // Lower than built-in providers
    }
}

impl PluginProvider for PluginWrapper {
    fn plugin(&self) -> &Plugin {
        &self.plugin
    }

    fn execute_hook(&self, _hook: &str, _context: &str) -> Result<String, String> {
        // This would call the plugin's entry script
        // For now, return not implemented
        Err(format!("Hook execution not implemented for plugin {}", self.plugin.name))
    }

    fn execute_tool(&self, _tool_name: &str, _args: &Value) -> Result<String, String> {
        // This would call the plugin's entry script with tool_execute hook
        Err(format!("Tool execution not implemented for plugin {}", self.plugin.name))
    }

    fn tool_schemas(&self) -> Vec<Value> {
        self.plugin.tools.iter().map(|tool_name| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": format!("plugin_{}", tool_name),
                    "description": format!("Plugin tool: {} (from {})", tool_name, self.plugin.name),
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "args": {
                                "type": "object",
                                "description": "Arguments to pass to the plugin tool"
                            }
                        },
                        "required": []
                    }
                }
            })
        }).collect()
    }
}

/// Registry for plugin providers
pub type PluginProviderRegistry = crate::providers::ProviderRegistry<dyn PluginProvider>;

/// Create plugin provider registry from PluginManager
pub fn plugin_registry_from_manager(manager: &crate::plugins::manager::PluginManager) -> PluginProviderRegistry {
    let mut registry = PluginProviderRegistry::new();
    for plugin in manager.list() {
        if plugin.enabled {
            registry.register(Box::new(PluginWrapper::new(plugin.clone())));
        }
    }
    registry
}