use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A citation reference for tracking sources and attributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Unique identifier for the citation
    pub id: String,
    /// Title of the cited work
    pub title: String,
    /// Author or creator
    pub author: String,
    /// Source URL or location
    pub source: String,
    /// Date accessed or published
    pub date: Option<String>,
    /// Context or quote from the source
    pub context: Option<String>,
    /// Whether this citation has been verified
    pub verified: bool,
}

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
    /// Citations managed by this plugin
    #[serde(default)]
    pub citations: Vec<Citation>,
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
    InstallError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::NotFound(n) => write!(f, "plugin not found: {}", n),
            PluginError::InvalidManifest(m) => write!(f, "invalid manifest: {}", m),
            PluginError::LoadError(e) => write!(f, "load error: {}", e),
            PluginError::HookError(e) => write!(f, "hook error: {}", e),
            PluginError::InstallError(e) => write!(f, "install error: {}", e),
        }
    }
}

impl std::error::Error for PluginError {}

/// Aggregate plugin statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub hook_count: usize,
    pub tool_count: usize,
    pub citation_count: usize,
}

impl std::fmt::Display for PluginStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plugins: {} total, {} enabled, {} disabled | {} hooks, {} tools, {} citations",
            self.total,
            self.enabled,
            self.disabled,
            self.hook_count,
            self.tool_count,
            self.citation_count
        )
    }
}
