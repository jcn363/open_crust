use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProviderType {
    Ollama,
    OpenRouter,
    OpenAI,
    Gemini,
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
    // Add more as needed, following the pattern
}

fn default_leader() -> String { "ctrl+x".to_string() }
fn default_app_exit() -> String { "ctrl+c,ctrl+d".to_string() }
fn default_input_submit() -> String { "return".to_string() }

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

fn default_color_bg() -> String { "#1e1e1e".to_string() }
fn default_color_fg() -> String { "#ffffff".to_string() }
fn default_color_accent() -> String { "#007acc".to_string() }
fn default_color_border() -> String { "#333333".to_string() }

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
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            leader: default_leader(),
            app_exit: default_app_exit(),
            input_submit: default_input_submit(),
        }
    }
}

fn default_permissions() -> std::collections::HashMap<String, PermissionRule> {
    let mut map = std::collections::HashMap::new();
    map.insert("*".to_string(), PermissionRule::Action(PermissionAction::Ask));
    map
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: ProviderType::Ollama,
            model: "llama3".to_string(),
            ollama_url: Some("http://localhost:11434".to_string()),
            openrouter_key: None,
            mcp: std::collections::HashMap::new(),
            lsp: std::collections::HashMap::new(),
            instructions: Vec::new(),
            permission: default_permissions(),
            allowed_domains: Vec::new(),
            tui: None,
            theme: None,
            openai_key: None,
            gemini_api_key: None,
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
                    eprintln!("Warning: OpenRouter provider selected but openrouter_key is not set");
                }
            }
            ProviderType::OpenAI => {
                if self.openai_key.is_none() || self.openai_key.as_ref().unwrap().is_empty() {
                    eprintln!("Warning: OpenAI provider selected but openai_key is not set");
                }
            }
            ProviderType::Gemini => {
                if self.gemini_api_key.is_none() || self.gemini_api_key.as_ref().unwrap().is_empty() {
                    eprintln!("Warning: Gemini provider selected but gemini_api_key is not set");
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
                eprintln!("Warning: LSP server '{}' has no file extensions specified", name);
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
}

/// Basic validation for hex color strings
fn validate_hex_color(color: &str, field_name: &str) {
    if !color.starts_with('#') || (color.len() != 7 && color.len() != 4) {
        eprintln!("Warning: {} should be a hex color (e.g., #1e1e1e), got: {}", field_name, color);
    }
}
