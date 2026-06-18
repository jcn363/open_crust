//! Configuration system: loading, serialization, and type definitions
//!
//! Defines all configuration types including provider settings, model aliases,
//! MCP/LSP server configs, keybindings, theme, subagent configuration, and
//! permission rules. Loads from and saves to a TOML file. Central to every
//! subsystem's initialization.

mod types;
mod vllm_config;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub use types::*;

fn default_permissions() -> std::collections::HashMap<String, PermissionRule> {
    let mut map = std::collections::HashMap::new();
    map.insert(
        "*".to_string(),
        PermissionRule::Action(PermissionAction::Ask),
    );
    map
}

fn default_auto_summarize() -> bool {
    true
}

fn default_audit_retention() -> u64 {
    90
}

fn default_audit_max_size() -> u64 {
    10_485_760
}

fn default_token_budget_max() -> u32 {
    1_000_000
}

/// Top-level configuration for OpenCrust
///
/// Loaded from a TOML file. Contains provider settings, model aliases,
/// MCP/LSP server configs, UI theme, keybindings, permission rules,
/// auto-refresh settings, and subagent configuration.
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

    // --- Context summarization configuration ---
    /// Whether to automatically summarize context when approaching token limit
    #[serde(default = "default_auto_summarize")]
    pub auto_summarize: bool,

    // --- Compliance / Audit configuration ---
    #[serde(default)]
    pub compliance_mode: bool,
    #[serde(default)]
    pub compliance_log_path: Option<String>,
    #[serde(default = "default_audit_retention")]
    pub audit_retention_days: u64,
    #[serde(default = "default_audit_max_size")]
    pub audit_max_size_bytes: u64,

    // --- Plugin system configuration ---
    #[serde(default)]
    pub plugins: PluginConfig,

    // --- Token budget configuration ---
    /// Maximum tokens per session (default: 1_000_000)
    #[serde(default = "default_token_budget_max")]
    pub token_budget_max_tokens: u32,
    /// Whether token budget enforcement is enabled (default: true)
    #[serde(default = "default_true")]
    pub token_budget_enabled: bool,
    /// Provider fallback chain (e.g., ["openai", "anthropic", "groq"])
    #[serde(default)]
    pub fallback_chain: Vec<String>,

    // --- Role-based access control ---
    /// User role for permission templates (admin, developer, reviewer)
    #[serde(default)]
    pub role: crate::permissions::Role,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: ProviderType::OpenRouter,
            model: "openrouter/free".to_string(),
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
            auto_summarize: true,
            compliance_mode: false,
            compliance_log_path: None,
            audit_retention_days: default_audit_retention(),
            audit_max_size_bytes: default_audit_max_size(),
            plugins: PluginConfig::default(),
            token_budget_max_tokens: default_token_budget_max(),
            token_budget_enabled: true,
            fallback_chain: Vec::new(),
            role: crate::permissions::Role::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/opencrust");
        let config_path = config_dir.join("config.json");

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<Config>(&content) {
                    Ok(config) => {
                        let _ = config.validate();
                        config
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config ({}), using defaults", e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read config ({}), using defaults", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/opencrust");
        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("Warning: Failed to create config dir: {}", e);
        }
        let config_path = config_dir.join("config.json");
        let content = match serde_json::to_string(self) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to serialize config: {}", e);
                return;
            }
        };
        if let Err(e) = fs::write(&config_path, &content) {
            eprintln!("Warning: Failed to write config: {}", e);
        }
    }

    /// Validate configuration and return warnings/errors
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut warnings = Vec::new();

        // Check provider-specific requirements
        match self.provider {
            ProviderType::Ollama => {
                if self.ollama_url.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push("Ollama provider selected but ollama_url is not set".to_string());
                }
            }
            ProviderType::OpenRouter => {
                if self.openrouter_key.as_ref().is_none_or(|v| v.is_empty()) {
                    let is_free = self.model.to_lowercase().contains("free");
                    if !is_free {
                        warnings.push(
                            "OpenRouter provider selected but openrouter_key is not set"
                                .to_string(),
                        );
                    }
                }
            }
            ProviderType::OpenAI => {
                if self.openai_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push("OpenAI provider selected but openai_key is not set".to_string());
                }
            }
            ProviderType::Gemini => {
                if self.gemini_api_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings
                        .push("Gemini provider selected but gemini_api_key is not set".to_string());
                }
            }
            ProviderType::Mistral => {
                if self.mistral_api_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push(
                        "Mistral provider selected but mistral_api_key is not set".to_string(),
                    );
                }
            }
            ProviderType::Anthropic => {
                if self.anthropic_api_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push(
                        "Anthropic provider selected but anthropic_api_key is not set".to_string(),
                    );
                }
            }
            ProviderType::Groq => {
                if self.groq_api_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push("Groq provider selected but groq_api_key is not set".to_string());
                }
            }
            ProviderType::TogetherAi => {
                if self.together_api_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push(
                        "TogetherAi provider selected but together_api_key is not set".to_string(),
                    );
                }
            }
            ProviderType::Replicate => {
                if self.replicate_api_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push(
                        "Replicate provider selected but replicate_api_key is not set".to_string(),
                    );
                }
            }
            ProviderType::DeepSeek => {
                if self.deepseek_api_key.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings.push(
                        "DeepSeek provider selected but deepseek_api_key is not set".to_string(),
                    );
                }
            }
            ProviderType::LocalAi => {
                if self.localai_url.as_ref().is_none_or(|v| v.is_empty()) {
                    warnings
                        .push("LocalAi provider selected but localai_url is not set".to_string());
                }
            }
        }

        // Check model is not empty
        if self.model.is_empty() {
            warnings.push("model is not specified".to_string());
        }

        // Validate MCP configurations
        for (name, mcp) in &self.mcp {
            if mcp.command.is_empty() {
                warnings.push(format!("MCP server '{}' has empty command", name));
            }
        }

        // Validate LSP configurations
        for (name, lsp) in &self.lsp {
            if lsp.command.is_empty() {
                warnings.push(format!("LSP server '{}' has empty command", name));
            }
            if lsp.extensions.is_empty() {
                warnings.push(format!(
                    "LSP server '{}' has no file extensions specified",
                    name
                ));
            }
        }

        // Validate theme colors (basic check for hex format)
        if let Some(theme) = &self.theme {
            validate_hex_color(&theme.background, "theme.background", &mut warnings);
            validate_hex_color(&theme.foreground, "theme.foreground", &mut warnings);
            validate_hex_color(&theme.accent, "theme.accent", &mut warnings);
            validate_hex_color(&theme.border, "theme.border", &mut warnings);
        }

        if warnings.is_empty() {
            Ok(())
        } else {
            for w in &warnings {
                eprintln!("Warning: {}", w);
            }
            Err(warnings)
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
    #[expect(dead_code, reason = "model resolution for subagents")]
    pub fn resolve_subagent_model(&self, agent_type: Option<&str>) -> (ProviderType, String) {
        // Check environment variable first
        if let Ok(env_model) = std::env::var("OPENCRUST_SUBAGENT_MODEL")
            && !env_model.is_empty()
        {
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
    pub(crate) fn parse_model_string(&self, model_str: &str) -> (ProviderType, String) {
        // Check if it's a model alias first
        if let Some(alias) = self.model_aliases.get(model_str) {
            return (alias.provider.clone(), alias.model_id.clone());
        }

        // Check "provider/model" format
        if let Some((provider_str, model_id)) = model_str.split_once('/') {
            let provider = provider_str
                .parse()
                .unwrap_or_else(|_| self.provider.clone());
            return (provider, model_id.to_string());
        }

        // If just a model name without provider, assume current provider
        (self.provider.clone(), model_str.to_string())
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
    // Requires GITHUB_TOKEN environment variable to be set by user
    mcp.insert(
        "github".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            environment: None,
            enabled: false,
        },
    );

    // Brave Search - Web research
    // Requires BRAVE_API_KEY environment variable to be set by user
    mcp.insert(
        "brave-search".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-brave-search".to_string(),
            ],
            environment: None,
            enabled: false,
        },
    );

    // PostgreSQL - Database queries
    // Requires DATABASE_URL environment variable to be set by user
    mcp.insert(
        "postgres".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-postgres".to_string(),
            ],
            environment: None,
            enabled: false,
        },
    );

    // Filesystem - Enhanced file operations
    // Requires ALLOWED_DIRS environment variable to be set by user
    mcp.insert(
        "filesystem".to_string(),
        McpConfig {
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
            ],
            environment: None,
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
            command: vec![
                "yolo".to_string(),
                "detect".to_string(),
                "predict".to_string(),
            ],
            environment: None,
            enabled: false, // Requires ultralytics installation: pip install ultralytics
        },
    );

    mcp
}
