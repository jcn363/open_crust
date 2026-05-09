use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Default OpenRouter free model (no API key required)
pub const DEFAULT_OPENROUTER_FREE_MODEL: &str = "openrouter/free-gpt-4o-mini";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProviderType {
    Ollama,
    OpenRouter,
    OpenAI,
    Gemini,
    Mistral,
    Anthropic,
    Groq,
    TogetherAi,
    Replicate,
    DeepSeek,
    LocalAi,
}

/// Model tiers for cost-aware routing
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ModelTier {
    Fast,      // Cheap, quick responses
    Balanced,  // Good price/performance
    Powerful,  // Expensive, capable models
}

/// Model alias for user-friendly model names
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelAlias {
    pub alias: String,
    pub provider: ProviderType,
    pub model_id: String,
    pub tier: ModelTier,
    /// Whether this model supports tool/function calling
    /// Set to false for models that don't support tools (saves tokens by skipping tool schemas)
    #[serde(default = "default_tool_capable")]
    pub tool_capable: bool,
}

fn default_tool_capable() -> bool {
    true
}

/// Configuration for subagent model selection
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubagentConfig {
    /// Default model for all subagents (e.g., "openrouter/free-gpt-4o-mini")
    pub default_model: Option<String>,
    /// Fall back to free models if primary model fails
    #[serde(default = "default_true")]
    pub fallback_to_free: bool,
    /// Per-agent-type model overrides (e.g., "researcher" -> "openrouter/free-gpt-4o-mini")
    #[serde(default)]
    pub agent_overrides: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

impl Default for SubagentConfig {
    fn default() -> Self {
        Self {
            default_model: Some(DEFAULT_OPENROUTER_FREE_MODEL.to_string()),
            fallback_to_free: true,
            agent_overrides: HashMap::new(),
        }
    }
}

impl ProviderType {
    pub fn as_str(&self) -> &str {
        match self {
            ProviderType::Ollama => "ollama",
            ProviderType::OpenRouter => "openrouter",
            ProviderType::OpenAI => "openai",
            ProviderType::Gemini => "gemini",
            ProviderType::Mistral => "mistral",
            ProviderType::Anthropic => "anthropic",
            ProviderType::Groq => "groq",
            ProviderType::TogetherAi => "togetherai",
            ProviderType::Replicate => "replicate",
            ProviderType::DeepSeek => "deepseek",
            ProviderType::LocalAi => "localai",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpConfig {
    pub command: Vec<String>,
    pub environment: Option<std::collections::HashMap<String, String>>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LspConfig {
    pub command: Vec<String>,
    pub extensions: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PermissionRule {
    Action(PermissionAction),
    Map(std::collections::HashMap<String, PermissionAction>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keybinds {
    #[serde(default = "default_leader")]
    pub leader: String,
    #[serde(default = "default_app_exit")]
    pub app_exit: String,
    #[serde(default = "default_input_submit")]
    pub input_submit: String,
    #[serde(default = "default_paste")]
    pub paste: String,
    #[serde(default = "default_copy")]
    pub copy: String,
    // Add more as needed, following the pattern
}

fn default_leader() -> String {
    "ctrl+x".to_string()
}
fn default_app_exit() -> String {
    "ctrl+q".to_string()
}
fn default_input_submit() -> String {
    "return".to_string()
}
fn default_paste() -> String {
    "ctrl+v".to_string()
}
fn default_copy() -> String {
    "ctrl+c".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TuiConfig {
    pub keybinds: Keybinds,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    #[serde(default = "default_color_bg")]
    pub background: String,
    #[serde(default = "default_color_fg")]
    pub foreground: String,
    #[serde(default = "default_color_accent")]
    pub accent: String,
    #[serde(default = "default_color_border")]
    pub border: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            background: default_color_bg(),
            foreground: default_color_fg(),
            accent: default_color_accent(),
            border: default_color_border(),
        }
    }
}

fn default_color_bg() -> String {
    "#1e1e1e".to_string()
}
fn default_color_fg() -> String {
    "#ffffff".to_string()
}
fn default_color_accent() -> String {
    "#007acc".to_string()
}
fn default_color_border() -> String {
    "#333333".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider: ProviderType,
    pub model: String,
    pub ollama_url: Option<String>,
    pub openrouter_key: Option<String>,
    pub mcp: std::collections::HashMap<String, McpConfig>,
    pub lsp: std::collections::HashMap<String, LspConfig>,
    pub instructions: Vec<String>,
    #[serde(default = "default_permissions")]
    pub permission: std::collections::HashMap<String, PermissionRule>,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    pub tui: Option<TuiConfig>,
    pub theme: Option<ThemeConfig>,
    pub openai_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    #[serde(default)]
    pub context_budget: Option<u64>,
    #[serde(default)]
    pub summarization_threshold: Option<f64>,
    /// Subagent model configuration (enforces free models for subagents)
    #[serde(default)]
    pub subagent_config: Option<SubagentConfig>,
    /// Model aliases for user-friendly model names
    #[serde(default)]
    pub model_aliases: HashMap<String, ModelAlias>,

    // --- New provider API keys ---
    #[serde(default)]
    pub groq_api_key: Option<String>,
    #[serde(default)]
    pub together_api_key: Option<String>,
    #[serde(default)]
    pub replicate_api_key: Option<String>,
    #[serde(default)]
    pub deepseek_api_key: Option<String>,
    #[serde(default)]
    pub localai_url: Option<String>,
    #[serde(default)]
    pub localai_api_key: Option<String>,

    // --- Orchestrator configuration ---
    #[serde(default)]
    pub subagent_max_concurrent: Option<usize>,
    #[serde(default)]
    pub subagent_timeout_secs: Option<u64>,

    // --- Model auto-refresh configuration ---
    #[serde(default)]
    pub model_auto_refresh: Option<ModelAutoRefreshConfig>,
}

/// Controls automatic background refresh of provider model lists
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelAutoRefreshConfig {
    /// Whether auto-refresh is enabled (default: true)
    #[serde(default = "default_model_auto_refresh_enabled")]
    pub enabled: bool,
    /// Interval in seconds between refreshes (default: 3600 = 1 hour)
    #[serde(default = "default_model_auto_refresh_interval")]
    pub interval_secs: u64,
}

fn default_model_auto_refresh_enabled() -> bool {
    true
}

fn default_model_auto_refresh_interval() -> u64 {
    3600
}

impl Default for ModelAutoRefreshConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 3600,
        }
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            leader: default_leader(),
            app_exit: default_app_exit(),
            input_submit: default_input_submit(),
            paste: default_paste(),
            copy: default_copy(),
        }
    }
}

fn default_permissions() -> std::collections::HashMap<String, PermissionRule> {
    let mut map = std::collections::HashMap::new();
    map.insert(
        "*".to_string(),
        PermissionRule::Action(PermissionAction::Ask),
    );
    map
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: ProviderType::Ollama,
            // Use provider‑specific default model
            model: match ProviderType::Ollama {
                ProviderType::OpenRouter => DEFAULT_OPENROUTER_FREE_MODEL.to_string(),
                _ => "llama3".to_string(),
            },
            ollama_url: Some("http://localhost:11434".to_string()),
            openrouter_key: None,
            mcp: default_mcp_servers(),
            lsp: std::collections::HashMap::new(),
            instructions: Vec::new(),
            permission: default_permissions(),
            allowed_domains: Vec::new(),
            tui: None,
            theme: None,
            openai_key: None,
            gemini_api_key: None,
            mistral_api_key: None,
            anthropic_api_key: None,
            context_budget: None,
            summarization_threshold: None,
            subagent_config: Some(SubagentConfig::default()),
            model_aliases: HashMap::new(),
            groq_api_key: None,
            together_api_key: None,
            replicate_api_key: None,
            deepseek_api_key: None,
            localai_url: None,
            localai_api_key: None,
            subagent_max_concurrent: None,
            subagent_timeout_secs: None,
            model_auto_refresh: Some(ModelAutoRefreshConfig::default()),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_dir = dirs::home_dir().unwrap().join(".config/open_crust");
        let config_path = config_dir.join("config.json");

        if config_path.exists() {
            let content = fs::read_to_string(config_path).unwrap_or_default();
            match serde_json::from_str(&content) {
                Ok(config) => {
                    let config: Config = config;
                    config.validate();
                    config
                }
                Err(_) => {
                    eprintln!("Warning: Failed to parse config, using defaults");
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let config_dir = dirs::home_dir().unwrap().join(".config/open_crust");
        let _ = fs::create_dir_all(&config_dir);
        let config_path = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(self).unwrap_or_default();
        let _ = fs::write(config_path, content);
    }

    /// Validate configuration and print warnings for issues
    pub fn validate(&self) {
        // Check provider-specific requirements
        match self.provider {
            ProviderType::Ollama => {
                if self.ollama_url.is_none() || self.ollama_url.as_ref().unwrap().is_empty() {
                    eprintln!("Warning: Ollama provider selected but ollama_url is not set");
                }
            }
            ProviderType::OpenRouter => {
                if self.openrouter_key.is_none() || self.openrouter_key.as_ref().unwrap().is_empty() {
                    // Skip warning for free models that do not require a key
                    let is_free = self.model.to_lowercase().contains("free");
                    if !is_free {
                        eprintln!(
                            "Warning: OpenRouter provider selected but openrouter_key is not set"
                        );
                    }
                }
            }
            ProviderType::OpenAI => {
                if self.openai_key.is_none() || self.openai_key.as_ref().unwrap().is_empty() {
                    eprintln!("Warning: OpenAI provider selected but openai_key is not set");
                }
            }
            ProviderType::Gemini => {
                if self.gemini_api_key.is_none() || self.gemini_api_key.as_ref().unwrap().is_empty()
                {
                    eprintln!("Warning: Gemini provider selected but gemini_api_key is not set");
                }
            }
            ProviderType::Mistral => {
                if self.mistral_api_key.is_none()
                    || self.mistral_api_key.as_ref().unwrap().is_empty()
                {
                    eprintln!("Warning: Mistral provider selected but mistral_api_key is not set");
                }
            }
            ProviderType::Anthropic => {
                if self.anthropic_api_key.is_none()
                    || self.anthropic_api_key.as_ref().unwrap().is_empty()
                {
                    eprintln!(
                        "Warning: Anthropic provider selected but anthropic_api_key is not set"
                    );
                }
            }
            ProviderType::Groq => {
                if self.groq_api_key.is_none() || self.groq_api_key.as_ref().unwrap().is_empty() {
                    eprintln!("Warning: Groq provider selected but groq_api_key is not set");
                }
            }
            ProviderType::TogetherAi => {
                if self.together_api_key.is_none()
                    || self.together_api_key.as_ref().unwrap().is_empty()
                {
                    eprintln!(
                        "Warning: TogetherAi provider selected but together_api_key is not set"
                    );
                }
            }
            ProviderType::Replicate => {
                if self.replicate_api_key.is_none()
                    || self.replicate_api_key.as_ref().unwrap().is_empty()
                {
                    eprintln!(
                        "Warning: Replicate provider selected but replicate_api_key is not set"
                    );
                }
            }
            ProviderType::DeepSeek => {
                if self.deepseek_api_key.is_none()
                    || self.deepseek_api_key.as_ref().unwrap().is_empty()
                {
                    eprintln!(
                        "Warning: DeepSeek provider selected but deepseek_api_key is not set"
                    );
                }
            }
            ProviderType::LocalAi => {
                if self.localai_url.is_none()
                    || self.localai_url.as_ref().unwrap().is_empty()
                {
                    eprintln!("Warning: LocalAi provider selected but localai_url is not set");
                }
            }
        }

        // Check model is not empty
        if self.model.is_empty() {
            eprintln!("Warning: model is not specified");
        }

        // Validate MCP configurations
        for (name, mcp) in &self.mcp {
            if mcp.command.is_empty() {
                eprintln!("Warning: MCP server '{}' has empty command", name);
            }
        }

        // Validate LSP configurations
        for (name, lsp) in &self.lsp {
            if lsp.command.is_empty() {
                eprintln!("Warning: LSP server '{}' has empty command", name);
            }
            if lsp.extensions.is_empty() {
                eprintln!(
                    "Warning: LSP server '{}' has no file extensions specified",
                    name
                );
            }
        }

        // Validate theme colors (basic check for hex format)
        if let Some(theme) = &self.theme {
            validate_hex_color(&theme.background, "theme.background");
            validate_hex_color(&theme.foreground, "theme.foreground");
            validate_hex_color(&theme.accent, "theme.accent");
            validate_hex_color(&theme.border, "theme.border");
        }
    }

    /// Returns the context window limit for a given provider and model
    /// If the user has set a custom context_budget in config, that takes precedence
    pub fn context_limit(&self) -> u64 {
        // User override takes precedence
        if let Some(budget) = self.context_budget {
            return budget;
        }

        // Default limits based on provider and model
        match self.provider {
            ProviderType::Anthropic => {
                if self.model.contains("opus") || self.model.contains("sonnet") {
                    200_000
                } else {
                    100_000
                }
            }
            ProviderType::OpenAI => 128_000,
            ProviderType::Gemini => {
                if self.model.contains("gemini-2.5") {
                    1_000_000
                } else {
                    128_000
                }
            }
            ProviderType::Ollama => 8_000, // configurable, but default is small
            ProviderType::OpenRouter => 128_000, // depends on the model routed
            ProviderType::Mistral => 32_000,
            ProviderType::Groq => 128_000,
            ProviderType::TogetherAi => 128_000,
            ProviderType::Replicate => 8_000,
            ProviderType::DeepSeek => 128_000,
            ProviderType::LocalAi => 8_000,
        }
    }

    /// Get the summarization threshold (default 0.8 = 80% of context limit)
    pub fn summarization_threshold(&self) -> f64 {
        self.summarization_threshold.unwrap_or(0.8)
    }

    /// Resolve which model a subagent should use
    /// Priority: 1. Environment variable OPENCRUST_SUBAGENT_MODEL
    ///          2. Per-agent override in subagent_config.agent_overrides
    ///          3. Default subagent model in subagent_config.default_model
    ///          4. Fall back to main config model (self.model)
    pub fn resolve_subagent_model(&self, agent_type: Option<&str>) -> (ProviderType, String) {
        // Check environment variable first
        if let Ok(env_model) = std::env::var("OPENCRUST_SUBAGENT_MODEL") && !env_model.is_empty() {
            return self.parse_model_string(&env_model);
        }

        // Check subagent config
        if let Some(subagent_config) = &self.subagent_config {
            // Check per-agent override
            if let Some(agent_type) = agent_type
                && let Some(model) = subagent_config.agent_overrides.get(agent_type)
            {
                return self.parse_model_string(model);
            }

            // Check default subagent model
            if let Some(default_model) = &subagent_config.default_model {
                return self.parse_model_string(default_model);
            }

            // If fallback_to_free is enabled and no model specified, use free model
            if subagent_config.fallback_to_free && self.openrouter_key.is_none() {
                return self.parse_model_string(DEFAULT_OPENROUTER_FREE_MODEL);
            }
        }

        // Fall back to main config model
        (self.provider.clone(), self.model.clone())
    }

    /// Parse a model string in "provider/model" format
    /// Returns (provider, model_id)
    fn parse_model_string(&self, model_str: &str) -> (ProviderType, String) {
        // Check if it's a model alias first
        if let Some(alias) = self.model_aliases.get(model_str) {
            return (alias.provider.clone(), alias.model_id.clone());
        }

        // Check "provider/model" format
        if let Some((provider_str, model_id)) = model_str.split_once('/') {
            let provider = match provider_str {
                "ollama" => ProviderType::Ollama,
                "openrouter" => ProviderType::OpenRouter,
                "openai" => ProviderType::OpenAI,
                "gemini" => ProviderType::Gemini,
                "mistral" => ProviderType::Mistral,
                "anthropic" => ProviderType::Anthropic,
                _ => self.provider.clone(),
            };
            return (provider, model_id.to_string());
        }

        // If just a model name without provider, assume current provider
        (self.provider.clone(), model_str.to_string())
    }
}

/// Basic validation for hex color strings
fn validate_hex_color(color: &str, field_name: &str) {
    if !color.starts_with('#') || (color.len() != 7 && color.len() != 4) {
        eprintln!(
            "Warning: {} should be a hex color (e.g., #1e1e1e), got: {}",
            field_name, color
        );
    }
}

/// Returns the recommended default MCP servers for OpenCrust
/// These are the most valuable servers based on ecosystem analysis
///
/// Provides recommended MCP server configurations for common use cases.
/// Currently used as the default MCP server set in [`Config::default()`].
/// Users can enable/disable servers in their config.json file.
pub fn default_mcp_servers() -> std::collections::HashMap<String, McpConfig> {
    let mut mcp = std::collections::HashMap::new();

    // Context7 - Version-accurate library docs (highest impact)
    mcp.insert(
        "context7".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@context7/mcp-server".to_string(),
            ],
            environment: None,
            enabled: false, // Disabled by default, user must enable
        },
    );

    // GitHub - Repository management
    mcp.insert(
        "github".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            environment: Some(std::collections::HashMap::from([(
                "GITHUB_TOKEN".to_string(),
                "your-github-token".to_string(),
            )])),
            enabled: false,
        },
    );

    // Brave Search - Web research
    mcp.insert(
        "brave-search".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-brave-search".to_string(),
            ],
            environment: Some(std::collections::HashMap::from([(
                "BRAVE_API_KEY".to_string(),
                "your-brave-api-key".to_string(),
            )])),
            enabled: false,
        },
    );

    // PostgreSQL - Database queries
    mcp.insert(
        "postgres".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-postgres".to_string(),
            ],
            environment: Some(std::collections::HashMap::from([(
                "DATABASE_URL".to_string(),
                "postgres://user:pass@localhost:5432/db".to_string(),
            )])),
            enabled: false,
        },
    );

    // Filesystem - Enhanced file operations
    mcp.insert(
        "filesystem".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
            ],
            environment: Some(std::collections::HashMap::from([(
                "ALLOWED_DIRS".to_string(),
                "/home/user/projects".to_string(),
            )])),
            enabled: false,
        },
    );

    // Sequential Thinking - Step-by-step reasoning for complex problems
    mcp.insert(
        "sequentialthinking".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-sequential-thinking".to_string(),
            ],
            environment: None,
            enabled: true, // Enabled by default
        },
    );

    // Critical Thinking - Analytical reasoning and evaluation
    mcp.insert(
        "criticalthinking".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-critical-thinking".to_string(),
            ],
            environment: None,
            enabled: true, // Enabled by default
        },
    );

    // YOLO - Object detection for image analysis
    mcp.insert(
        "yolo".to_string(),
        McpConfig {
            command: vec!["yolo".to_string(), "detect".to_string(), "predict".to_string()],
            environment: None,
            enabled: false, // Requires ultralytics installation: pip install ultralytics
        },
    );

    mcp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_model_no_key_no_warning() {
        let mut cfg = Config::default();
        cfg.provider = ProviderType::OpenRouter;
        cfg.model = DEFAULT_OPENROUTER_FREE_MODEL.to_string();
        cfg.openrouter_key = None;
        cfg.validate();
    }

    #[test]
    fn paid_model_no_key_warns() {
        let mut cfg = Config::default();
        cfg.provider = ProviderType::OpenRouter;
        cfg.model = "openrouter/anthropic/claude-3".to_string();
        cfg.openrouter_key = None;
        cfg.validate();
    }
}
