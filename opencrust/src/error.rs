/// Unified error type for OpenCrust
#[derive(Debug, thiserror::Error)]
pub enum OpenCrustError {
    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    // HTTP errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    // Config errors
    #[error("Configuration error: {0}")]
    Config(String),

    // LLM errors
    #[error("LLM error: {0}")]
    Llm(String),

    // Tool errors
    #[error("Tool error: {tool}: {message}")]
    Tool { tool: String, message: String },

    // Permission errors
    #[error("Permission denied: {0}")]
    Permission(String),

    // Session errors
    #[error("Session error: {0}")]
    Session(String),

    // MCP errors
    #[error("MCP error: {0}")]
    Mcp(String),

    // LSP errors
    #[error("LSP error: {0}")]
    Lsp(String),

    // Skill errors
    #[error("Skill error: {0}")]
    Skill(String),

    // Plugin errors
    #[error("Plugin error: {0}")]
    Plugin(String),

    // Path errors
    #[error("Invalid path: {0}")]
    Path(String),

    // Security errors
    #[error("Security violation: {0}")]
    Security(String),

    // Provider errors
    #[error("Provider error: {0}")]
    Provider(String),

    // Parse errors
    #[error("Parse error: {0}")]
    Parse(String),

    // Generic with context
    #[error("{0}")]
    Other(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for OpenCrustError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        OpenCrustError::Other(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for OpenCrustError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        OpenCrustError::Other(e.to_string())
    }
}

/// Result type alias for OpenCrust operations
pub type Result<T> = std::result::Result<T, OpenCrustError>;
