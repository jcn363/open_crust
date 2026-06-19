//! Integration tests for ProviderType parsing, display, and validation.

use opencrust::config::ProviderType;

/// Every provider variant round-trips via Display and FromStr.
#[test]
fn all_providers_display_and_parse() {
    let cases = [
        (ProviderType::Ollama, "ollama"),
        (ProviderType::OpenAI, "openai"),
        (ProviderType::Gemini, "gemini"),
        (ProviderType::Anthropic, "anthropic"),
        (ProviderType::Mistral, "mistral"),
        (ProviderType::Groq, "groq"),
        (ProviderType::TogetherAi, "togetherai"),
        (ProviderType::Replicate, "replicate"),
        (ProviderType::OpenRouter, "openrouter"),
        (ProviderType::DeepSeek, "deepseek"),
        (ProviderType::LocalAi, "localai"),
        (ProviderType::Unsloth, "unsloth"),
    ];

    for (variant, expected_str) in &cases {
        let display = variant.to_string();
        assert_eq!(display, *expected_str, "Display for {variant:?}");

        let parsed: ProviderType = display
            .parse()
            .unwrap_or_else(|_| panic!("FromStr for {expected_str}"));
        assert_eq!(parsed, *variant, "Round-trip for {expected_str}");
    }
}

/// Unknown strings produce a parse error.
#[test]
fn unknown_provider_parse_fails() {
    let result: Result<ProviderType, String> = "nonexistent_provider".parse();
    assert!(result.is_err(), "unknown provider should fail to parse");
}

/// Empty string fails to parse.
#[test]
fn empty_provider_parse_fails() {
    let result: Result<ProviderType, String> = "".parse();
    assert!(result.is_err(), "empty string should fail to parse");
}

/// ProviderType is Send + Sync (required for async usage).
#[test]
fn provider_type_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<ProviderType>();
    assert_sync::<ProviderType>();
}

/// ProviderType has Debug, Clone, PartialEq, Eq trait impls.
#[test]
fn provider_type_has_required_traits() {
    fn assert_debug<T: std::fmt::Debug>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_partial_eq<T: PartialEq>() {}
    fn assert_eq_trait<T: Eq>() {}

    assert_debug::<ProviderType>();
    assert_clone::<ProviderType>();
    assert_partial_eq::<ProviderType>();
    assert_eq_trait::<ProviderType>();
}
