use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProviderType {
    Ollama,
    OpenRouter,
    OpenAI,
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
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_dir = dirs::home_dir().unwrap().join(".config/open_crust");
        let config_path = config_dir.join("config.json");
        
        if config_path.exists() {
            let content = fs::read_to_string(config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Self::default())
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
}
