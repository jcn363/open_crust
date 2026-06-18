//! Centralized configuration management following vLLM VllmConfig pattern
//!
//! This module implements a single, unified configuration object that follows
//! the vLLM pattern of having all configuration in one place. It provides
//! validation, serialization, and a clean API for accessing configuration values.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::types::*;
use crate::permissions::Role;

/// Centralized application configuration following vLLM VllmConfig pattern
///
/// This is the single source of truth for all application configuration.
/// It follows the vLLM pattern having all configuration in one object
/// with validation at startup.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct VllmConfig {
    // Core application settings
    pub app: AppConfig,
    
    // LLM provider configuration
    pub providers: ProviderConfig,
    
    // MCP (Model Context Protocol) configuration
    pub mcp: VllmMcpConfig,
    
    // LSP (Language Server Protocol) configuration
    pub lsp: VllmLspConfig,
    
    // UI and theme configuration
    pub ui: UiConfig,
    
    // Permission and security configuration
    pub permissions: PermissionConfig,
    
    // Plugin system configuration
    pub plugins: VllmPluginConfig,
    
    // Token budget and rate limiting
    pub budget: BudgetConfig,
    
    // Subagent configuration
    pub subagents: SubagentConfig,
    
    // Desktop integration
    pub desktop: DesktopConfig,
    
    // Logging and monitoring
    pub logging: LoggingConfig,
    
    // Advanced features
    pub advanced: AdvancedConfig,
}

/// Core application settings
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub log_level: String,
    pub enable_telemetry: bool,
    pub max_concurrent_tasks: usize,
    pub task_timeout_secs: u64,
}

/// LLM provider configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct ProviderConfig {
    pub default_provider: ProviderType,
    pub models: HashMap<String, ModelConfig>,
    pub api_keys: HashMap<String, String>,
    pub endpoints: HashMap<String, String>,
    pub rate_limits: HashMap<String, RateLimitConfig>,
    pub fallback_chain: Vec<String>,
}

/// Individual model configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct ModelConfig {
    pub provider: ProviderType,
    pub model_id: String,
    pub tier: ModelTier,
    pub context_limit: u64,
    pub temperature: f32,
    pub max_tokens: u32,
    pub tool_capable: bool,
    pub description: String,
}

/// Rate limiting configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub burst_capacity: u32,
}

/// MCP configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct VllmMcpConfig {
    pub servers: HashMap<String, VllmMcpServerConfig>,
    pub auto_discover: bool,
    pub discovery_timeout_secs: u64,
    pub max_servers: usize,
}

/// Individual MCP server configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct VllmMcpServerConfig {
    pub command: Vec<String>,
    pub environment: Option<HashMap<String, String>>,
    pub enabled: bool,
    pub auto_start: bool,
    pub restart_policy: RestartPolicy,
    pub health_check_interval_secs: u64,
}

/// LSP configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct VllmLspConfig {
    pub servers: HashMap<String, VllmLspServerConfig>,
    pub auto_start: bool,
    pub max_servers: usize,
}

/// Individual LSP server configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct VllmLspServerConfig {
    pub command: Vec<String>,
    pub extensions: Vec<String>,
    pub environment: Option<HashMap<String, String>>,
    pub disabled: bool,
    pub initialization_options: Option<serde_json::Value>,
}

/// UI configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct UiConfig {
    pub theme: ThemeConfig,
    pub keybinds: Keybinds,
    pub layout: LayoutConfig,
    pub panels: PanelConfig,
    pub animations: bool,
    pub status_bar: bool,
}

/// Layout configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct LayoutConfig {
    pub sidebar_width: u16,
    pub panel_split_ratio: f32,
    pub tab_bar_height: u16,
    pub input_height: u16,
    pub status_height: u16,
}

/// Panel configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct PanelConfig {
    pub max_panels: usize,
    pub auto_hide: bool,
    pub focus_follows_mouse: bool,
}

/// Permission configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct PermissionConfig {
    pub default_role: Role,
    pub templates: HashMap<String, PermissionTemplate>,
    pub rules: HashMap<String, PermissionRule>,
    pub audit_enabled: bool,
    pub audit_retention_days: u64,
    pub audit_max_size_bytes: u64,
}

/// Permission template
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct PermissionTemplate {
    pub role: Role,
    pub rules: HashMap<String, PermissionAction>,
    pub description: String,
}

/// Plugin configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct VllmPluginConfig {
    pub enabled: bool,
    pub search_paths: Vec<String>,
    pub disabled_plugins: Vec<String>,
    pub auto_load: bool,
    pub max_plugins: usize,
}

/// Budget configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct BudgetConfig {
    pub token_budget: TokenBudgetConfig,
    pub request_budget: RequestBudgetConfig,
    pub cost_tracking: bool,
    pub alerting_enabled: bool,
}

/// Token budget configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct TokenBudgetConfig {
    pub max_tokens_per_session: u32,
    pub max_tokens_per_day: u32,
    pub reset_on_session_end: bool,
    pub warning_threshold_percent: f32,
}

/// Request budget configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct RequestBudgetConfig {
    pub max_requests_per_minute: u32,
    pub max_requests_per_hour: u32,
    pub max_requests_per_day: u32,
}

/// Desktop integration configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct DesktopConfig {
    pub notifications: NotificationConfig,
    pub file_picker: FilePickerConfig,
    pub menu_bar: MenuBarConfig,
    pub auto_detect: bool,
}

/// Notification configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct NotificationConfig {
    pub enabled: bool,
    pub timeout_secs: u64,
    pub max_notifications: usize,
    pub desktop_notifications: bool,
}

/// File picker configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct FilePickerConfig {
    pub enabled: bool,
    pub default_directory: Option<PathBuf>,
    pub allow_multiple: bool,
}

/// Menu bar configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct MenuBarConfig {
    pub enabled: bool,
    pub show_app_info: bool,
    pub show_preferences: bool,
    pub show_services: bool,
}

/// Logging configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<PathBuf>,
    pub max_file_size_mb: u64,
    pub max_files: usize,
    pub enable_console: bool,
    pub enable_file: bool,
}

/// Advanced configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
pub struct AdvancedConfig {
    pub experimental_features: bool,
    pub debug_mode: bool,
    pub profile_performance: bool,
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

/// Restart policy for MCP servers
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
    OnError,
}

impl VllmConfig {
    /// Create a new VllmConfig with default values
    #[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
    pub fn new() -> Self {
        Self {
            app: AppConfig::default(),
            providers: ProviderConfig::default(),
            mcp: VllmMcpConfig::default(),
            lsp: VllmLspConfig::default(),
            ui: UiConfig::default(),
            permissions: PermissionConfig::default(),
            plugins: VllmPluginConfig::default(),
            budget: BudgetConfig::default(),
            subagents: SubagentConfig::default(),
            desktop: DesktopConfig::default(),
            logging: LoggingConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
    
    /// Load configuration from file
    #[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
    pub fn load(path: Option<PathBuf>) -> Result<Self, String> {
        let config_path = path.unwrap_or_else(|| {
            let mut config_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            config_dir.push(".config/opencrust");
            config_dir.push("config.toml");
            config_dir
        });
        
        if !config_path.exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: VllmConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        config.validate().map_err(|warnings| warnings.join("; "))?;
        Ok(config)
    }
    
    /// Save configuration to file
    #[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
    pub fn save(&self, path: Option<PathBuf>) -> Result<(), String> {
        let config_path = path.unwrap_or_else(|| {
            let mut config_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            config_dir.push(".config/opencrust");
            config_dir.push("config.toml");
            config_dir
        });
        
        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        let content = toml::to_string(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        std::fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        Ok(())
    }
    
    /// Validate configuration
    #[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut warnings = Vec::new();
        
        // Validate core configuration
        if self.app.name.is_empty() {
            warnings.push("App name is empty".to_string());
        }
        
        if self.app.version.is_empty() {
            warnings.push("App version is empty".to_string());
        }
        
        // Validate provider configuration
        if self.providers.default_provider == ProviderType::OpenRouter {
            if let Some(key) = self.providers.api_keys.get("openrouter") {
                if key.is_empty() {
                    warnings.push("OpenRouter API key is empty".to_string());
                }
            } else {
                warnings.push("OpenRouter provider selected but no API key configured".to_string());
            }
        }
        
        // Validate MCP configuration
        for (name, server) in &self.mcp.servers {
            if server.command.is_empty() {
                warnings.push(format!("MCP server '{}' has empty command", name));
            }
            if server.enabled && server.auto_start {
                warnings.push(format!("MCP server '{}' is both enabled and auto-started", name));
            }
        }
        
        // Validate LSP configuration
        for (name, server) in &self.lsp.servers {
            if server.command.is_empty() {
                warnings.push(format!("LSP server '{}' has empty command", name));
            }
        }
        
        // Validate theme colors
        validate_hex_color(&self.ui.theme.background, "ui.theme.background", &mut warnings);
        validate_hex_color(&self.ui.theme.foreground, "ui.theme.foreground", &mut warnings);
        validate_hex_color(&self.ui.theme.accent, "ui.theme.accent", &mut warnings);
        validate_hex_color(&self.ui.theme.border, "ui.theme.border", &mut warnings);
        
        if warnings.is_empty() {
            Ok(())
        } else {
            Err(warnings)
        }
    }
    
    /// Get a reference to the configuration for a specific provider
    #[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
    pub fn get_provider_config(&self, provider: &ProviderType) -> Option<&ModelConfig> {
        self.providers.models.values().find(|m| &m.provider == provider)
    }
    
    /// Get the default model for a provider
    #[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
    pub fn get_default_model(&self, provider: &ProviderType) -> Option<&ModelConfig> {
        self.providers
            .models
            .values()
            .find(|m| m.provider == *provider && m.tier == ModelTier::Balanced)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            name: "OpenCrust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: home_dir.join(".local/share/opencrust"),
            config_dir: home_dir.join(".config/opencrust"),
            log_level: "info".to_string(),
            enable_telemetry: false,
            max_concurrent_tasks: 10,
            task_timeout_secs: 300,
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        let mut models = HashMap::new();
        
        models.insert(
            "default".to_string(),
            ModelConfig {
                provider: ProviderType::OpenRouter,
                model_id: "openrouter/free".to_string(),
                tier: ModelTier::Balanced,
                context_limit: 128_000,
                temperature: 0.7,
                max_tokens: 4096,
                tool_capable: true,
                description: "Default OpenRouter free model".to_string(),
            },
        );
        
        models.insert(
            "fast".to_string(),
            ModelConfig {
                provider: ProviderType::OpenRouter,
                model_id: "openrouter/meta-llama/llama-3.1-8b-instruct".to_string(),
                tier: ModelTier::Fast,
                context_limit: 8_000,
                temperature: 0.5,
                max_tokens: 2048,
                tool_capable: true,
                description: "Fast Llama 3.1 model".to_string(),
            },
        );
        
        let mut api_keys = HashMap::new();
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            api_keys.insert("openrouter".to_string(), key);
        }
        
        let mut endpoints = HashMap::new();
        endpoints.insert("openrouter".to_string(), "https://openrouter.ai/api/v1".to_string());
        
        Self {
            default_provider: ProviderType::OpenRouter,
            models,
            api_keys,
            endpoints,
            rate_limits: HashMap::new(),
            fallback_chain: vec!["openrouter".to_string(), "openai".to_string()],
        }
    }
}

impl Default for VllmMcpConfig {
    fn default() -> Self {
        let mut servers = HashMap::new();
        
        // Add default MCP servers
        servers.insert(
            "sequentialthinking".to_string(),
            VllmMcpServerConfig {
                command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-sequential-thinking".to_string()],
                environment: None,
                enabled: true,
                auto_start: false,
                restart_policy: RestartPolicy::OnFailure,
                health_check_interval_secs: 30,
            },
        );
        
        Self {
            servers,
            auto_discover: true,
            discovery_timeout_secs: 60,
            max_servers: 20,
        }
    }
}

impl Default for VllmLspConfig {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
            auto_start: false,
            max_servers: 10,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            keybinds: Keybinds::default(),
            layout: LayoutConfig::default(),
            panels: PanelConfig::default(),
            animations: true,
            status_bar: true,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: 30,
            panel_split_ratio: 0.6,
            tab_bar_height: 2,
            input_height: 2,
            status_height: 1,
        }
    }
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            max_panels: 10,
            auto_hide: false,
            focus_follows_mouse: false,
        }
    }
}

impl Default for PermissionConfig {
    fn default() -> Self {
        let mut templates = HashMap::new();
        
        // Developer template
        templates.insert(
            "developer".to_string(),
            PermissionTemplate {
                role: Role::Developer,
                rules: HashMap::new(), // Will be populated from default permissions
                description: "Developer role with standard permissions".to_string(),
            },
        );
        
        // Reviewer template
        templates.insert(
            "reviewer".to_string(),
            PermissionTemplate {
                role: Role::Reviewer,
                rules: HashMap::new(),
                description: "Reviewer role with read-only permissions".to_string(),
            },
        );
        
        Self {
            default_role: Role::Developer,
            templates,
            rules: HashMap::new(),
            audit_enabled: true,
            audit_retention_days: 90,
            audit_max_size_bytes: 10_485_760,
        }
    }
}

impl Default for VllmPluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            search_paths: vec!["~/.opencrust/plugins".to_string()],
            disabled_plugins: Vec::new(),
            auto_load: true,
            max_plugins: 50,
        }
    }
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            token_budget: TokenBudgetConfig::default(),
            request_budget: RequestBudgetConfig::default(),
            cost_tracking: true,
            alerting_enabled: false,
        }
    }
}

impl Default for TokenBudgetConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_session: 1_000_000,
            max_tokens_per_day: 10_000_000,
            reset_on_session_end: true,
            warning_threshold_percent: 0.9,
        }
    }
}

impl Default for RequestBudgetConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
            max_requests_per_day: 10000,
        }
    }
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            notifications: NotificationConfig::default(),
            file_picker: FilePickerConfig::default(),
            menu_bar: MenuBarConfig::default(),
            auto_detect: true,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_secs: 5,
            max_notifications: 10,
            desktop_notifications: true,
        }
    }
}

impl Default for FilePickerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_directory: None,
            allow_multiple: false,
        }
    }
}

impl Default for MenuBarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_app_info: true,
            show_preferences: true,
            show_services: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file: Some(home_dir.join(".local/share/opencrust/logs/opencrust.log")),
            max_file_size_mb: 100,
            max_files: 5,
            enable_console: true,
            enable_file: true,
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            experimental_features: false,
            debug_mode: false,
            profile_performance: false,
            enable_metrics: false,
            metrics_port: 9090,
            custom_settings: HashMap::new(),
        }
    }
}

#[allow(dead_code, reason = "New configuration system - will be integrated in future phases")]
fn validate_hex_color(color: &str, field_name: &str, warnings: &mut Vec<String>) {
    let valid = color.starts_with('#')
        && (color.len() == 7 || color.len() == 4)
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if !valid {
        warnings.push(format!(
            "{} should be a hex color (e.g., #1e1e1e), got: {}",
            field_name, color
        ));
    }
}