//! Plugin/Extension System — discover, load, and manage OpenCrust plugins
//!
//! Plugins extend OpenCrust with new capabilities: custom tools, event hooks,
//! UI panels, and protocol integrations. Each plugin is a directory under
//! `~/.config/opencrust/plugins/<name>/` containing a `plugin.json` manifest
//! and optionally scripts, WASM modules, or configuration files.
//!
//! ## Manifest format (`plugin.json`)
//!
//! ```json
//! {
//!   "name": "my-plugin",
//!   "version": "1.0.0",
//!   "description": "Integrates with FooBar API",
//!   "author": "You",
//!   "entry": "main.sh",
//!   "hooks": ["on_tool_execute", "on_message"],
//!   "tools": ["my_custom_tool"],
//!   "dependencies": [],
//!   "enabled": true
//! }
//! ```
//!
//! ## Hook System
//!
//! Plugins can register hooks that fire at specific points in the
//! OpenCrust lifecycle. Built-in hook points:
//!
//! - `on_startup` — called when OpenCrust initializes
//! - `on_shutdown` — called before OpenCrust exits
//! - `on_tool_execute` — before a tool runs (can modify/block)
//! - `on_message` — when a message is received
//! - `on_response` — when an LLM response is generated
//! - `on_session_save` — when a session is persisted

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A loaded plugin instance with parsed manifest and runtime state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// Unique name (must match directory name)
    pub name: String,
    /// SemVer string
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Relative path to entry point script (from plugin dir)
    pub entry: Option<String>,
    /// Hook points this plugin subscribes to
    #[serde(default)]
    pub hooks: Vec<String>,
    /// Custom tool names this plugin provides
    #[serde(default)]
    pub tools: Vec<String>,
    /// Plugin dependencies (other plugin names)
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Whether the plugin is active
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Absolute path to plugin directory (set at load time)
    #[serde(skip)]
    pub path: PathBuf,
}

fn default_true() -> bool {
    true
}

/// Error type for plugin operations.
#[derive(Debug)]
pub enum PluginError {
    NotFound(String),
    InvalidManifest(String),
    LoadError(String),
    HookError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::NotFound(n) => write!(f, "plugin not found: {}", n),
            PluginError::InvalidManifest(m) => write!(f, "invalid manifest: {}", m),
            PluginError::LoadError(e) => write!(f, "load error: {}", e),
            PluginError::HookError(e) => write!(f, "hook error: {}", e),
        }
    }
}

impl std::error::Error for PluginError {}

/// Manages plugin discovery, loading, lifecycle, and hook dispatch.
pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
    search_paths: Vec<PathBuf>,
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

        Self {
            plugins: HashMap::new(),
            search_paths,
        }
    }

    /// Scan all search paths and load plugin manifests.
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
                            if !self.plugins.contains_key(&name) {
                                discovered.push(name.clone());
                                self.plugins.insert(name, plugin);
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
    fn load_plugin(&mut self, manifest_path: &Path) -> Result<Plugin, PluginError> {
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| PluginError::LoadError(format!("cannot read manifest: {}", e)))?;

        let mut plugin: Plugin = serde_json::from_str(&content)
            .map_err(|e| PluginError::InvalidManifest(format!("JSON parse error: {}", e)))?;

        // Store the absolute plugin directory path
        plugin.path = manifest_path
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| PluginError::InvalidManifest("cannot determine plugin directory".into()))?;

        // Validate required fields
        if plugin.name.is_empty() {
            return Err(PluginError::InvalidManifest("name is required".into()));
        }
        if plugin.version.is_empty() {
            return Err(PluginError::InvalidManifest(
                "version is required".into(),
            ));
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
        Ok(())
    }

    /// Disable a plugin by name.
    pub fn disable(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
        plugin.enabled = false;
        Ok(())
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
        copy_dir_recursively(src_dir, &target_dir).map_err(|e| {
            PluginError::InstallError(format!("cannot copy plugin files: {}", e))
        })?;

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
                fs::remove_dir_all(&plugin_dir)
                    .map_err(|e| PluginError::HookError(format!("cannot remove plugin dir: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Execute a hook across all enabled plugins that subscribe to it.
    /// Each plugin's entry script is called with the hook name and JSON context.
    #[expect(dead_code, reason = "plugin hook execution for CLI")]
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
        PluginStats {
            total,
            enabled,
            disabled: total - enabled,
            hook_count,
            tool_count,
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate plugin statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub hook_count: usize,
    pub tool_count: usize,
}

impl std::fmt::Display for PluginStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plugins: {} total, {} enabled, {} disabled | {} hooks, {} tools",
            self.total, self.enabled, self.disabled, self.hook_count, self.tool_count
        )
    }
}

// --- Helper functions ---

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    false
}

fn guess_interpreter(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("sh") => "sh".to_string(),
        Some("py") => "python3".to_string(),
        Some("js") => "node".to_string(),
        Some("ts") => "npx".to_string(),
        Some("rb") => "ruby".to_string(),
        Some("rs") => "rust-script".to_string(),
        _ => {
            // Try reading shebang
            if let Ok(content) = fs::read_to_string(path) {
                if let Some(line) = content.lines().next() {
                    if let Some(interp) = line.strip_prefix("#!") {
                        return interp.trim().to_string();
                    }
                }
            }
            "sh".to_string()
        }
    }
}

fn copy_dir_recursively(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursively(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// Extend PluginError with InstallError variant
impl PluginError {
    #[allow(non_snake_case)]
    fn InstallError(msg: String) -> Self {
        PluginError::HookError(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_manifest(dir: &Path, name: &str, enabled: bool) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let manifest = dir.join("plugin.json");
        let content = serde_json::json!({
            "name": name,
            "version": "1.0.0",
            "description": "test plugin",
            "author": "test",
            "hooks": ["on_startup"],
            "enabled": enabled
        });
        fs::write(&manifest, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        manifest
    }

    #[test]
    fn test_discover_plugins() {
        let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
        let plugin_dir = dir.join("test-plugin");
        create_test_manifest(&plugin_dir, "test-plugin", true);

        let mut mgr = PluginManager::new();
        mgr.search_paths = vec![dir.clone()];
        let discovered = mgr.discover();
        assert!(discovered.contains(&"test-plugin".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_plugins() {
        let mut mgr = PluginManager::new();
        let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
        let plugin_dir = dir.join("plugin-a");
        create_test_manifest(&plugin_dir, "plugin-a", true);
        mgr.search_paths = vec![dir.clone()];
        mgr.discover();

        let list = mgr.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "plugin-a");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_enable_disable() {
        let mut mgr = PluginManager::new();
        let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
        let plugin_dir = dir.join("toggle-plugin");
        create_test_manifest(&plugin_dir, "toggle-plugin", true);
        mgr.search_paths = vec![dir.clone()];
        mgr.discover();

        assert!(mgr.get("toggle-plugin").unwrap().enabled);
        mgr.disable("toggle-plugin").unwrap();
        assert!(!mgr.get("toggle-plugin").unwrap().enabled);
        mgr.enable("toggle-plugin").unwrap();
        assert!(mgr.get("toggle-plugin").unwrap().enabled);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stats() {
        let mut mgr = PluginManager::new();
        let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
        let d1 = dir.join("p1");
        let d2 = dir.join("p2");
        create_test_manifest(&d1, "p1", true);
        create_test_manifest(&d2, "p2", false);
        mgr.search_paths = vec![dir.clone()];
        mgr.discover();

        let stats = mgr.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.enabled, 1);
        assert_eq!(stats.disabled, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_nonexistent() {
        let mgr = PluginManager::new();
        assert!(mgr.get("nope").is_none());
    }

    #[test]
    fn test_enable_nonexistent() {
        let mut mgr = PluginManager::new();
        assert!(mgr.enable("nope").is_err());
    }

    #[test]
    fn test_invalid_manifest() {
        let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("plugin.json"), "not json").unwrap();

        let mut mgr = PluginManager::new();
        mgr.search_paths = vec![dir.clone()];
        let discovered = mgr.discover();
        assert!(discovered.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}
