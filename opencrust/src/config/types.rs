//! Configuration type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Default OpenRouter free model (no API key required)
pub const DEFAULT_OPENROUTER_FREE_MODEL: &str = "openrouter/free";

/// Supported LLM provider backends
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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
    /// Unsloth Studio — OpenAI-compatible local inference API
    /// Supports 500+ models with 2x faster training, 70% less VRAM
    /// Default URL: http://localhost:8000/v1
    Unsloth,
    /// Azure OpenAI — Enterprise Azure deployments
    AzureOpenAi,
    /// GitHub Copilot — Use existing Copilot subscription
    GitHubCopilot,
    /// Amazon Bedrock — AWS-hosted models
    Bedrock,
    /// Vertex AI — Google Cloud Vertex AI
    VertexAi,
    /// Perplexity — Perplexity AI models
    Perplexity,
    /// Cohere — Cohere models
    Cohere,
    /// Cerebras — Cerebras inference
    Cerebras,
    /// Alibaba Cloud — Alibaba Cloud models
    AlibabaCloud,
    /// Venice AI — Venice AI models
    VeniceAi,
    /// NVIDIA — NVIDIA NIM models
    Nvidia,
    /// Fireworks AI — Fireworks AI models
    FireworksAi,
    /// SambaNova — SambaNova models
    SambaNova,
    /// OctoAI — OctoAI models
    OctoAi,
    /// Anyscale — Anyscale models
    Anyscale,
    /// Lambda Labs — Lambda Labs models
    LambdaLabs,
    /// RunPod — RunPod models
    RunPod,
    /// Modal — Modal models
    Modal,
    /// Replicate — Replicate models (already exists)
    /// Hugging Face — Hugging Face Inference API
    HuggingFace,
    /// LM Studio — Local model management
    LMStudio,
    /// Text Generation Inference — TGI endpoints
    TGI,
    /// vLLM — vLLM OpenAI-compatible endpoints
    VLLM,
    /// Ollama — Local models (already exists)
    /// Custom OpenAI-compatible endpoint
    CustomOpenAi,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderType::Ollama => write!(f, "ollama"),
            ProviderType::OpenRouter => write!(f, "openrouter"),
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::Gemini => write!(f, "gemini"),
            ProviderType::Mistral => write!(f, "mistral"),
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::Groq => write!(f, "groq"),
            ProviderType::TogetherAi => write!(f, "togetherai"),
            ProviderType::Replicate => write!(f, "replicate"),
            ProviderType::DeepSeek => write!(f, "deepseek"),
            ProviderType::LocalAi => write!(f, "localai"),
            ProviderType::Unsloth => write!(f, "unsloth"),
            ProviderType::AzureOpenAi => write!(f, "azure"),
            ProviderType::GitHubCopilot => write!(f, "github-copilot"),
            ProviderType::Bedrock => write!(f, "bedrock"),
            ProviderType::VertexAi => write!(f, "vertex"),
            ProviderType::Perplexity => write!(f, "perplexity"),
            ProviderType::Cohere => write!(f, "cohere"),
            ProviderType::Cerebras => write!(f, "cerebras"),
            ProviderType::AlibabaCloud => write!(f, "alibaba"),
            ProviderType::VeniceAi => write!(f, "venice"),
            ProviderType::Nvidia => write!(f, "nvidia"),
            ProviderType::FireworksAi => write!(f, "fireworks"),
            ProviderType::SambaNova => write!(f, "sambanova"),
            ProviderType::OctoAi => write!(f, "octoai"),
            ProviderType::Anyscale => write!(f, "anyscale"),
            ProviderType::LambdaLabs => write!(f, "lambdalabs"),
            ProviderType::RunPod => write!(f, "runpod"),
            ProviderType::Modal => write!(f, "modal"),
            ProviderType::HuggingFace => write!(f, "huggingface"),
            ProviderType::LMStudio => write!(f, "lmstudio"),
            ProviderType::TGI => write!(f, "tgi"),
            ProviderType::VLLM => write!(f, "vllm"),
            ProviderType::CustomOpenAi => write!(f, "custom-openai"),
        }
    }
}

impl FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(ProviderType::Ollama),
            "openrouter" => Ok(ProviderType::OpenRouter),
            "openai" => Ok(ProviderType::OpenAI),
            "gemini" => Ok(ProviderType::Gemini),
            "mistral" => Ok(ProviderType::Mistral),
            "anthropic" => Ok(ProviderType::Anthropic),
            "groq" => Ok(ProviderType::Groq),
            "togetherai" | "together_ai" | "together-ai" => Ok(ProviderType::TogetherAi),
            "replicate" => Ok(ProviderType::Replicate),
            "deepseek" => Ok(ProviderType::DeepSeek),
            "localai" | "local_ai" | "local-ai" => Ok(ProviderType::LocalAi),
            "unsloth" => Ok(ProviderType::Unsloth),
            "azure" | "azure-openai" | "azure_openai" => Ok(ProviderType::AzureOpenAi),
            "github-copilot" | "github_copilot" | "copilot" => Ok(ProviderType::GitHubCopilot),
            "bedrock" | "aws-bedrock" | "aws_bedrock" => Ok(ProviderType::Bedrock),
            "vertex" | "vertex-ai" | "vertex_ai" => Ok(ProviderType::VertexAi),
            "perplexity" => Ok(ProviderType::Perplexity),
            "cohere" => Ok(ProviderType::Cohere),
            "cerebras" => Ok(ProviderType::Cerebras),
            "alibaba" | "alibaba-cloud" | "alibaba_cloud" => Ok(ProviderType::AlibabaCloud),
            "venice" | "venice-ai" | "venice_ai" => Ok(ProviderType::VeniceAi),
            "nvidia" | "nvidia-nim" | "nvidia_nim" => Ok(ProviderType::Nvidia),
            "fireworks" | "fireworks-ai" | "fireworks_ai" => Ok(ProviderType::FireworksAi),
            "sambanova" | "samba-nova" | "samba_nova" => Ok(ProviderType::SambaNova),
            "octoai" | "octo-ai" | "octo_ai" => Ok(ProviderType::OctoAi),
            "anyscale" => Ok(ProviderType::Anyscale),
            "lambdalabs" | "lambda-labs" | "lambda_labs" => Ok(ProviderType::LambdaLabs),
            "runpod" => Ok(ProviderType::RunPod),
            "modal" => Ok(ProviderType::Modal),
            "huggingface" | "hugging-face" | "hugging_face" => Ok(ProviderType::HuggingFace),
            "lmstudio" | "lm-studio" | "lm_studio" => Ok(ProviderType::LMStudio),
            "tgi" => Ok(ProviderType::TGI),
            "vllm" | "v-llm" | "v_llm" => Ok(ProviderType::VLLM),
            "custom-openai" | "custom_openai" | "custom" => Ok(ProviderType::CustomOpenAi),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

/// Model tiers for cost-aware routing
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ModelTier {
    Fast,     // Cheap, quick responses
    Balanced, // Good price/performance
    Powerful, // Expensive, capable models
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
    /// Default model for all subagents (e.g., "openrouter/free")
    pub default_model: Option<String>,
    /// Fall back to free models if primary model fails
    #[serde(default = "default_true")]
    pub fallback_to_free: bool,
    /// Per-agent-type model overrides (e.g., "researcher" -> "openrouter/free")
    #[serde(default)]
    pub agent_overrides: HashMap<String, String>,
}

pub(crate) fn default_true() -> bool {
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    "#131211".to_string()
}
fn default_color_fg() -> String {
    "#b6b5b4".to_string()
}
fn default_color_accent() -> String {
    "#f0eeee".to_string()
}
fn default_color_border() -> String {
    "#3c3b3a".to_string()
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

/// Plugin system configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    /// Whether plugin auto-discovery is enabled (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Plugin names to explicitly disable
    #[serde(default)]
    pub disabled_plugins: Vec<String>,
    /// Additional search paths for plugin directories
    #[serde(default)]
    pub search_paths: Vec<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            disabled_plugins: Vec::new(),
            search_paths: Vec::new(),
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

/// Basic validation for hex color strings
pub(crate) fn validate_hex_color(color: &str, field_name: &str, warnings: &mut Vec<String>) {
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
