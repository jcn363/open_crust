//! Integration tests for configuration loading, saving, and validation.
//!
//! These tests exercise the public API of the `opencrust` library via
//! real filesystem operations (temp directories).

use std::fs;

/// Config can be saved and reloaded without losing fields.
#[test]
fn config_save_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let config_path = dir.path().join("config.json");

    // Create a config with known values
    let mut config = opencrust::config::Config::default();
    config.provider = opencrust::config::ProviderType::Ollama;
    config.model = "llama3".into();
    config.ollama_url = Some("http://localhost:11434".into());

    // Save it
    let json = serde_json::to_string_pretty(&config).expect("serialize");
    fs::write(&config_path, &json).expect("write config");

    // Reload it
    let loaded_json = fs::read_to_string(&config_path).expect("read config");
    let loaded: opencrust::config::Config =
        serde_json::from_str(&loaded_json).expect("deserialize");

    assert_eq!(loaded.provider, opencrust::config::ProviderType::Ollama);
    assert_eq!(loaded.model, "llama3");
    assert_eq!(loaded.ollama_url, Some("http://localhost:11434".into()));
}

/// Default config is valid JSON and has expected default provider.
#[test]
fn config_default_is_valid() {
    let config = opencrust::config::Config::default();
    let json = serde_json::to_string_pretty(&config).expect("serialize");
    // Must parse back
    let parsed: opencrust::config::Config = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.provider, opencrust::config::ProviderType::OpenRouter);
}

/// All optional URL/API-key fields can be None without breaking serialization.
#[test]
fn config_optional_fields_can_be_none() {
    let config = opencrust::config::Config::default();
    let json = serde_json::to_string_pretty(&config).expect("serialize");
    // Optional fields should be absent or null, never crash
    assert!(!json.is_empty());
    // Reload: no error
    let _: opencrust::config::Config = serde_json::from_str(&json).expect("deserialize");
}

/// Config with all provider URL/key fields set round-trips correctly.
#[test]
fn config_all_provider_fields_roundtrip() {
    let config = opencrust::config::Config {
        ollama_url: Some("http://localhost:11434".into()),
        openai_key: Some("sk-test".into()),
        gemini_api_key: Some("gemini-test".into()),
        anthropic_api_key: Some("sk-ant-test".into()),
        mistral_api_key: Some("mistral-test".into()),
        groq_api_key: Some("gsk-test".into()),
        together_api_key: Some("together-test".into()),
        replicate_api_key: Some("r-test".into()),
        deepseek_api_key: Some("ds-test".into()),
        localai_url: Some("http://localhost:8080".into()),
        unsloth_url: Some("http://localhost:8000".into()),
        ..Default::default()
    };

    let json = serde_json::to_string_pretty(&config).expect("serialize");
    let loaded: opencrust::config::Config = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(loaded.ollama_url, Some("http://localhost:11434".into()));
    assert_eq!(loaded.openai_key, Some("sk-test".into()));
    assert_eq!(loaded.unsloth_url, Some("http://localhost:8000".into()));
    assert_eq!(loaded.localai_url, Some("http://localhost:8080".into()));
}

/// Config with all provider API keys set round-trips correctly.
#[test]
fn config_all_keys_roundtrip() {
    let config = opencrust::config::Config {
        openai_key: Some("sk-xxx".into()),
        gemini_api_key: Some("gemini-xxx".into()),
        mistral_api_key: Some("mistral-xxx".into()),
        anthropic_api_key: Some("sk-ant-xxx".into()),
        groq_api_key: Some("gsk-xxx".into()),
        together_api_key: Some("together-xxx".into()),
        replicate_api_key: Some("replicate-xxx".into()),
        deepseek_api_key: Some("ds-xxx".into()),
        localai_api_key: Some("localai-xxx".into()),
        unsloth_api_key: Some("unsloth-xxx".into()),
        ..Default::default()
    };

    let json = serde_json::to_string_pretty(&config).expect("serialize");
    let loaded: opencrust::config::Config = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(loaded.openai_key, Some("sk-xxx".into()));
    assert_eq!(loaded.unsloth_api_key, Some("unsloth-xxx".into()));
    assert_eq!(loaded.deepseek_api_key, Some("ds-xxx".into()));
}
