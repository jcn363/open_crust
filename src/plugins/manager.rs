use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::helpers::{copy_dir_recursively, guess_interpreter, is_executable};
use super::types::{Plugin, PluginError, PluginStats};

pub struct PluginManager {
    pub(crate) plugins: HashMap<String, Plugin>,
    pub(crate) search_paths: Vec<PathBuf>,
    pub(crate) state_file: PathBuf,
}

impl PluginManager {
    /// Create a new manager with default search paths.
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Local project plugins
        search_paths.push(PathBuf::from(".opencrust/plugins"));

        // User-level plugins
        if let Some(config_dir) = dirs::config_dir() {
            search_paths.push(config_dir.join("opencrust/plugins"));
        }

        // Global plugins (next to the binary)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                search_paths.push(parent.join("plugins"));
            }
        }

        let state_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencrust/plugins_state.json");

        Self {
            plugins: HashMap::new(),
            search_paths,
            state_file,
        }
    }

    /// Reload all plugins from disk, clearing the current map.
    #[expect(dead_code, reason = "public API for future plugin hot-reload")]
    pub fn reload(&mut self) {
        self.plugins.clear();
        self.discover();
    }

    pub fn discover(&mut self) -> Vec<String> {
        let mut discovered = Vec::new();
        // Collect all manifest paths first to avoid borrow conflicts
        let search_paths: Vec<_> = self.search_paths.clone();
        for search_path in &search_paths {
            if !search_path.exists() || !search_path.is_dir() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let manifest_path = path.join("plugin.json");
                    if !manifest_path.exists() {
                        continue;
                    }
                    match self.load_plugin(&manifest_path) {
                        Ok(plugin) => {
                            let name = plugin.name.clone();
                            if self.plugins.insert(name.clone(), plugin).is_none() {
                                discovered.push(name);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[Plugins] Failed to load {}: {}",
                                manifest_path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
        discovered
    }

    /// Load a single plugin from its manifest path.
    pub(crate) fn load_plugin(&mut self, manifest_path: &Path) -> Result<Plugin, PluginError> {
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| PluginError::LoadError(format!("cannot read manifest: {}", e)))?;

        let mut plugin: Plugin = serde_json::from_str(&content)
            .map_err(|e| PluginError::InvalidManifest(format!("JSON parse error: {}", e)))?;

        // Store the absolute plugin directory path
        plugin.path = manifest_path
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| {
                PluginError::InvalidManifest("cannot determine plugin directory".into())
            })?;

        // Validate required fields
        if plugin.name.is_empty() {
            return Err(PluginError::InvalidManifest("name is required".into()));
        }
        if plugin.version.is_empty() {
            return Err(PluginError::InvalidManifest("version is required".into()));
        }

        // If entry is specified, verify the file exists
        if let Some(ref entry) = plugin.entry {
            let entry_path = plugin.path.join(entry);
            if !entry_path.exists() {
                return Err(PluginError::InvalidManifest(format!(
                    "entry point '{}' not found",
                    entry
                )));
            }
        }

        Ok(plugin)
    }

    /// Return a list of all discovered plugins.
    pub fn list(&self) -> Vec<&Plugin> {
        let mut plugins: Vec<&Plugin> = self.plugins.values().collect();
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        plugins
    }

    /// Get a specific plugin by name.
    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }

    /// Get a mutable reference to a plugin.
    #[expect(dead_code, reason = "public API for future use")]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Plugin> {
        self.plugins.get_mut(name)
    }

    /// Enable a plugin by name.
    pub fn enable(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
        plugin.enabled = true;
        let _ = self.save_state();
        Ok(())
    }

    /// Disable a plugin by name.
    pub fn disable(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
        plugin.enabled = false;
        let _ = self.save_state();
        Ok(())
    }

    /// Save plugin enabled state to disk.
    pub fn save_state(&self) -> Result<(), PluginError> {
        let enabled: Vec<&str> = self
            .plugins
            .values()
            .filter(|p| p.enabled)
            .map(|p| p.name.as_str())
            .collect();
        let content = serde_json::to_string_pretty(&enabled)
            .map_err(|e| PluginError::LoadError(format!("cannot serialize state: {}", e)))?;
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PluginError::LoadError(format!("cannot create state dir: {}", e)))?;
        }
        fs::write(&self.state_file, content)
            .map_err(|e| PluginError::LoadError(format!("cannot write state: {}", e)))?;
        Ok(())
    }

    /// Load plugin enabled state from disk and apply it.
    pub fn load_state(&mut self) {
        if !self.state_file.exists() {
            return;
        }
        let content = match fs::read_to_string(&self.state_file) {
            Ok(c) => c,
            Err(_) => return,
        };
        let enabled: Vec<String> = match serde_json::from_str(&content) {
            Ok(e) => e,
            Err(_) => return,
        };
        // Apply loaded state: enable listed plugins, disable others
        for plugin in self.plugins.values_mut() {
            plugin.enabled = enabled.contains(&plugin.name);
        }
    }

    /// Install a plugin from a source directory by copying it into the user's
    /// plugin directory.
    pub fn install(&mut self, source: &Path) -> Result<String, PluginError> {
        let manifest_path = if source.is_dir() {
            source.join("plugin.json")
        } else {
            source.to_path_buf()
        };

        // Validate by loading first
        let plugin = self.load_plugin(&manifest_path)?;
        let name = plugin.name.clone();

        // Determine target directory
        let target_dir = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("opencrust/plugins").join(&name)
        } else {
            return Err(PluginError::InstallError(
                "cannot determine config directory".into(),
            ));
        };

        // Copy the source to target
        let src_dir = manifest_path
            .parent()
            .ok_or_else(|| PluginError::InvalidManifest("cannot determine source dir".into()))?;

        if target_dir.exists() {
            return Err(PluginError::InstallError(format!(
                "plugin '{}' already installed at {}",
                name,
                target_dir.display()
            )));
        }
        fs::create_dir_all(&target_dir).map_err(|e| {
            PluginError::InstallError(format!("cannot create target directory: {}", e))
        })?;

        // Recursively copy
        copy_dir_recursively(src_dir, &target_dir)
            .map_err(|e| PluginError::InstallError(format!("cannot copy plugin files: {}", e)))?;

        // Load the installed plugin
        let installed_manifest = target_dir.join("plugin.json");
        self.load_plugin(&installed_manifest)?;
        self.discover(); // re-scan to pick up the new plugin

        Ok(name)
    }

    /// Remove a plugin by name.
    pub fn remove(&mut self, name: &str) -> Result<(), PluginError> {
        self.plugins.remove(name);
        // Also remove from disk if in user plugin dir
        if let Some(config_dir) = dirs::config_dir() {
            let plugin_dir = config_dir.join("opencrust/plugins").join(name);
            if plugin_dir.exists() {
                fs::remove_dir_all(&plugin_dir).map_err(|e| {
                    PluginError::HookError(format!("cannot remove plugin dir: {}", e))
                })?;
            }
        }
        Ok(())
    }

    /// Execute a hook across all enabled plugins that subscribe to it.
    /// Each plugin's entry script is called with the hook name and JSON context.
    pub fn execute_hook(&self, hook: &str, context: &str) -> Vec<(String, Result<String, String>)> {
        let mut results = Vec::new();
        for plugin in self.plugins.values() {
            if !plugin.enabled {
                continue;
            }
            if !plugin.hooks.contains(&hook.to_string()) {
                continue;
            }
            let result = self.run_plugin_entry(plugin, hook, context);
            results.push((plugin.name.clone(), result));
        }
        results
    }

    /// Run a plugin's entry point script with the given hook name and JSON context.
    fn run_plugin_entry(
        &self,
        plugin: &Plugin,
        hook: &str,
        context: &str,
    ) -> Result<String, String> {
        let entry = match &plugin.entry {
            Some(e) => plugin.path.join(e),
            None => return Err("no entry point defined".into()),
        };

        if !entry.exists() {
            return Err(format!("entry point '{}' not found", entry.display()));
        }

        // Check if the entry is executable, if not try with appropriate interpreter
        let output = if is_executable(&entry) {
            Command::new(&entry)
                .arg(hook)
                .arg(context)
                .output()
                .map_err(|e| format!("failed to execute plugin: {}", e))?
        } else {
            // Try to determine interpreter from shebang or extension
            let interpreter = guess_interpreter(&entry);
            Command::new(&interpreter)
                .arg(&entry)
                .arg(hook)
                .arg(context)
                .output()
                .map_err(|e| format!("failed to execute plugin: {}", e))?
        };

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Return aggregate plugin statistics.
    pub fn stats(&self) -> PluginStats {
        let total = self.plugins.len();
        let enabled = self.plugins.values().filter(|p| p.enabled).count();
        let hook_count: usize = self.plugins.values().map(|p| p.hooks.len()).sum();
        let tool_count: usize = self.plugins.values().map(|p| p.tools.len()).sum();
        let citation_count: usize = self.plugins.values().map(|p| p.citations.len()).sum();
        PluginStats {
            total,
            enabled,
            disabled: total - enabled,
            hook_count,
            tool_count,
            citation_count,
        }
    }

    /// Get tool schemas for all enabled plugins that declare tools.
    /// Each plugin tool gets a generic schema that routes execution to the plugin's entry script.
    pub fn get_tool_schemas(&self) -> Vec<Value> {
        let mut schemas = Vec::new();
        for plugin in self.plugins.values() {
            if !plugin.enabled {
                continue;
            }
            for tool_name in &plugin.tools {
                schemas.push(serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": format!("plugin_{}", tool_name),
                        "description": format!("Plugin tool: {} (from {})", tool_name, plugin.name),
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
                }));
            }
        }
        schemas
    }

    /// Execute a plugin tool by name. Returns the tool output or an error.
    pub fn execute_tool(&self, tool_name: &str, args: &Value) -> Result<String, String> {
        // Find the plugin that owns this tool
        for plugin in self.plugins.values() {
            if !plugin.enabled {
                continue;
            }
            // Check if this tool belongs to this plugin (strip "plugin_" prefix)
            let bare_name = tool_name.strip_prefix("plugin_").unwrap_or(tool_name);
            if plugin.tools.iter().any(|t| t == bare_name) {
                let entry = match &plugin.entry {
                    Some(e) => plugin.path.join(e),
                    None => return Err(format!("plugin '{}' has no entry point", plugin.name)),
                };
                if !entry.exists() {
                    return Err(format!(
                        "plugin entry point '{}' not found",
                        entry.display()
                    ));
                }
                let context = serde_json::json!({
                    "tool": bare_name,
                    "args": args
                });
                let context_str = serde_json::to_string(&context).unwrap_or_default();
                return self.run_plugin_entry(plugin, "tool_execute", &context_str);
            }
        }
        Err(format!("no plugin owns tool '{}'", tool_name))
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
