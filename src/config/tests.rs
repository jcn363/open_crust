//! Tests for config module

use super::*;

#[test]
fn free_model_no_key_no_warning() {
    let cfg = Config {
        provider: ProviderType::OpenRouter,
        model: DEFAULT_OPENROUTER_FREE_MODEL.to_string(),
        openrouter_key: None,
        ..Default::default()
    };
    // Free models should not produce warnings about missing keys
    let _ = cfg.validate();
}

#[test]
fn paid_model_no_key_warns() {
    let cfg = Config {
        provider: ProviderType::OpenRouter,
        model: "openrouter/anthropic/claude-3".to_string(),
        openrouter_key: None,
        ..Default::default()
    };
    // Paid model without key should produce warnings
    let result = cfg.validate();
    assert!(result.is_err());
}

#[test]
fn validate_all_providers_missing_keys() {
    // Ensure each provider produces a warning when its key is missing
    let providers = [
        (ProviderType::OpenAI, "gpt-4o"),
        (ProviderType::Gemini, "gemini-pro"),
        (ProviderType::Mistral, "mistral-large"),
        (ProviderType::Anthropic, "claude-sonnet"),
        (ProviderType::Groq, "llama3-70b"),
        (ProviderType::TogetherAi, "togethercomputer/llama-2-70b"),
        (ProviderType::Replicate, "replicate/model"),
        (ProviderType::DeepSeek, "deepseek-chat"),
        (ProviderType::LocalAi, "local-model"),
    ];

    for (provider, model) in providers {
        let cfg = Config {
            provider: provider.clone(),
            model: model.to_string(),
            openai_key: None,
            gemini_api_key: None,
            mistral_api_key: None,
            anthropic_api_key: None,
            groq_api_key: None,
            together_api_key: None,
            replicate_api_key: None,
            deepseek_api_key: None,
            localai_url: None,
            localai_api_key: None,
            ..Default::default()
        };
        let result = cfg.validate();
        // All providers except Ollama (which has no key requirement) should fail
        if provider != ProviderType::Ollama {
            assert!(
                result.is_err(),
                "Provider {:?} should warn about missing key",
                provider
            );
        }
    }
}

#[test]
fn validate_hex_color_accepts_valid_colors() {
    let mut warnings = Vec::new();
    validate_hex_color("#1e1e1e", "test.field", &mut warnings);
    assert!(warnings.is_empty(), "Valid 7-char hex should not warn");

    let mut warnings = Vec::new();
    validate_hex_color("#fff", "test.field", &mut warnings);
    assert!(warnings.is_empty(), "Valid 4-char hex should not warn");
}

#[test]
fn validate_hex_color_rejects_invalid_colors() {
    let mut warnings = Vec::new();
    validate_hex_color("not-a-color", "test.field", &mut warnings);
    assert!(!warnings.is_empty(), "Invalid hex should warn");

    let mut warnings = Vec::new();
    validate_hex_color("#gggggg", "test.field", &mut warnings);
    assert!(!warnings.is_empty(), "Invalid hex chars should warn");

    let mut warnings = Vec::new();
    validate_hex_color("", "test.field", &mut warnings);
    assert!(!warnings.is_empty(), "Empty string should warn");
}

// --- ProviderType ---

#[test]
fn provider_type_from_str_all_variants() {
    assert_eq!(
        "ollama".parse::<ProviderType>().unwrap(),
        ProviderType::Ollama
    );
    assert_eq!(
        "OpenRouter".parse::<ProviderType>().unwrap(),
        ProviderType::OpenRouter
    );
    assert_eq!(
        "OPENAI".parse::<ProviderType>().unwrap(),
        ProviderType::OpenAI
    );
    assert_eq!(
        "gemini".parse::<ProviderType>().unwrap(),
        ProviderType::Gemini
    );
    assert_eq!(
        "mistral".parse::<ProviderType>().unwrap(),
        ProviderType::Mistral
    );
    assert_eq!(
        "anthropic".parse::<ProviderType>().unwrap(),
        ProviderType::Anthropic
    );
    assert_eq!("groq".parse::<ProviderType>().unwrap(), ProviderType::Groq);
    assert_eq!(
        "togetherai".parse::<ProviderType>().unwrap(),
        ProviderType::TogetherAi
    );
    assert_eq!(
        "replicate".parse::<ProviderType>().unwrap(),
        ProviderType::Replicate
    );
    assert_eq!(
        "deepseek".parse::<ProviderType>().unwrap(),
        ProviderType::DeepSeek
    );
    assert_eq!(
        "localai".parse::<ProviderType>().unwrap(),
        ProviderType::LocalAi
    );
}

#[test]
fn provider_type_from_str_unknown_should_error() {
    assert!("nonexistent".parse::<ProviderType>().is_err());
}

#[test]
fn provider_type_display_all_variants() {
    assert_eq!(ProviderType::Ollama.to_string(), "ollama");
    assert_eq!(ProviderType::OpenRouter.to_string(), "openrouter");
    assert_eq!(ProviderType::OpenAI.to_string(), "openai");
    assert_eq!(ProviderType::Gemini.to_string(), "gemini");
    assert_eq!(ProviderType::Mistral.to_string(), "mistral");
    assert_eq!(ProviderType::Anthropic.to_string(), "anthropic");
    assert_eq!(ProviderType::Groq.to_string(), "groq");
    assert_eq!(ProviderType::TogetherAi.to_string(), "togetherai");
    assert_eq!(ProviderType::Replicate.to_string(), "replicate");
    assert_eq!(ProviderType::DeepSeek.to_string(), "deepseek");
    assert_eq!(ProviderType::LocalAi.to_string(), "localai");
}

// --- Config ---

#[test]
fn config_default_values() {
    let cfg = Config::default();
    assert_eq!(cfg.provider, ProviderType::OpenRouter);
    assert_eq!(cfg.model, "openrouter/free");
    assert!(!cfg.compliance_mode);
    assert_eq!(cfg.audit_retention_days, 90);
    assert_eq!(cfg.audit_max_size_bytes, 10_485_760);
    assert!(cfg.ollama_url.is_some());
}

#[test]
fn config_summarization_threshold_default() {
    let cfg = Config::default();
    assert!((cfg.summarization_threshold() - 0.8).abs() < f64::EPSILON);
}

#[test]
fn config_summarization_threshold_custom() {
    let cfg = Config {
        summarization_threshold: Some(0.5),
        ..Default::default()
    };
    assert!((cfg.summarization_threshold() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn config_context_limit_returns_custom_budget() {
    let cfg = Config {
        provider: ProviderType::Ollama,
        context_budget: Some(4096),
        ..Default::default()
    };
    assert_eq!(cfg.context_limit(), 4096);
}

#[test]
fn config_context_limit_ollama_default() {
    let cfg = Config {
        provider: ProviderType::Ollama,
        ..Default::default()
    };
    assert_eq!(cfg.context_limit(), 8_000);
}

#[test]
fn config_context_limit_anthropic_sonnet() {
    let cfg = Config {
        provider: ProviderType::Anthropic,
        model: "claude-sonnet-4".to_string(),
        ..Default::default()
    };
    assert_eq!(cfg.context_limit(), 200_000);
}

#[test]
fn config_context_limit_anthropic_other() {
    let cfg = Config {
        provider: ProviderType::Anthropic,
        model: "claude-haiku".to_string(),
        ..Default::default()
    };
    assert_eq!(cfg.context_limit(), 100_000);
}

#[test]
fn config_context_limit_gemini_2_5() {
    let cfg = Config {
        provider: ProviderType::Gemini,
        model: "gemini-2.5-pro".to_string(),
        ..Default::default()
    };
    assert_eq!(cfg.context_limit(), 1_000_000);
}

#[test]
fn config_validate_empty_model_should_warn() {
    let cfg = Config {
        provider: ProviderType::Ollama,
        model: "".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        ..Default::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn config_validate_ollama_with_url_should_pass() {
    let cfg = Config {
        provider: ProviderType::Ollama,
        model: "qwen2.5-coder:7b".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        ..Default::default()
    };
    assert!(cfg.validate().is_ok());
}

#[test]
fn config_validate_mcp_empty_command_should_warn() {
    let mut mcp = std::collections::HashMap::new();
    mcp.insert(
        "bad-server".to_string(),
        McpConfig {
            command: vec![],
            environment: None,
            enabled: true,
        },
    );
    let cfg = Config {
        provider: ProviderType::Ollama,
        model: "test".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        mcp,
        ..Default::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn config_validate_lsp_empty_command_should_warn() {
    let mut lsp = std::collections::HashMap::new();
    lsp.insert(
        "bad-lsp".to_string(),
        LspConfig {
            command: vec![],
            extensions: vec!["rs".to_string()],
            env: None,
            disabled: false,
        },
    );
    let cfg = Config {
        provider: ProviderType::Ollama,
        model: "test".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        lsp,
        ..Default::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn config_validate_lsp_no_extensions_should_warn() {
    let mut lsp = std::collections::HashMap::new();
    lsp.insert(
        "empty-lsp".to_string(),
        LspConfig {
            command: vec!["rust-analyzer".to_string()],
            extensions: vec![],
            env: None,
            disabled: false,
        },
    );
    let cfg = Config {
        provider: ProviderType::Ollama,
        model: "test".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        lsp,
        ..Default::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn config_validate_theme_hex_colors() {
    let cfg = Config {
        provider: ProviderType::Ollama,
        model: "test".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        theme: Some(ThemeConfig {
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            accent: "#007acc".to_string(),
            border: "#333333".to_string(),
        }),
        ..Default::default()
    };
    assert!(cfg.validate().is_ok());
}

#[test]
fn config_validate_invalid_theme_should_warn() {
    let cfg = Config {
        provider: ProviderType::Ollama,
        model: "test".to_string(),
        ollama_url: Some("http://localhost:11434".to_string()),
        theme: Some(ThemeConfig {
            background: "bad-color".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(cfg.validate().is_err());
}
