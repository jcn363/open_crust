use super::*;

#[test]
fn test_bundled_defaults_non_empty() {
    let defaults = bundled_default_models();
    assert!(!defaults.is_empty());
    assert!(defaults.contains_key("openai"));
    assert!(defaults.contains_key("anthropic"));
    assert!(defaults.contains_key("openrouter"));
}

#[test]
fn test_cache_manager_create_load_missing() {
    let cm = CacheManager::new();
    let result = cm.load("nonexistent_provider", Duration::from_secs(3600));
    assert!(result.is_none());
}

#[test]
fn test_cache_manager_save_and_load() {
    let cm = CacheManager::new();
    let models = vec![ProviderModel {
        id: "test-model".to_string(),
        name: "Test Model".to_string(),
        provider: "test_provider".to_string(),
        description: "A test model".to_string(),
        context_length: 4096,
        tool_capable: true,
    }];
    cm.save("test_provider", models.clone());

    let loaded = cm.load("test_provider", Duration::from_secs(3600));
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().len(), 1);
}

#[test]
fn test_cache_stale_is_none() {
    let cm = CacheManager::new();
    // Manually write a cache entry with an old timestamp
    let entry = ModelCacheEntry {
        provider: "stale_provider".to_string(),
        models: vec![ProviderModel {
            id: "stale-model".to_string(),
            name: "Stale Model".to_string(),
            provider: "stale_provider".to_string(),
            description: String::new(),
            context_length: 4096,
            tool_capable: true,
        }],
        fetched_at: 1000, // ancient timestamp
    };
    let content = serde_json::to_string(&entry).unwrap();
    let path = cm.cache_dir.join("stale_provider.json");
    let _ = std::fs::create_dir_all(&cm.cache_dir);
    std::fs::write(&path, &content).unwrap();

    // With a very short max age, the cache should be considered stale
    let loaded = cm.load("stale_provider", Duration::from_secs(1));
    assert!(loaded.is_none());
}

#[test]
fn test_model_fetcher_urls() {
    let fetcher = ModelFetcher::new();
    assert_eq!(
        fetcher.build_url("openai", None),
        "https://api.openai.com/v1/models"
    );
    assert_eq!(
        fetcher.build_url("ollama", None),
        "http://localhost:11434/api/tags"
    );
    assert_eq!(
        fetcher.build_url("anthropic", None),
        "https://api.anthropic.com/v1/models"
    );
    assert_eq!(
        fetcher.build_url("gemini", None),
        "https://generativelanguage.googleapis.com/v1beta/models"
    );
}

#[test]
fn test_provider_model_default_tool_capable() {
    let model = ProviderModel {
        id: "test".to_string(),
        name: "Test".to_string(),
        provider: "test".to_string(),
        description: String::new(),
        context_length: 0,
        tool_capable: false,
    };
    assert!(!model.tool_capable);

    let serialized = serde_json::to_string(&model).unwrap();
    let deserialized: ProviderModel = serde_json::from_str(&serialized).unwrap();
    assert!(!deserialized.tool_capable);
}

#[test]
fn test_model_cache_entry_roundtrip() {
    let entry = ModelCacheEntry {
        provider: "test".to_string(),
        models: vec![ProviderModel {
            id: "m1".to_string(),
            name: "M1".to_string(),
            provider: "test".to_string(),
            description: "desc".to_string(),
            context_length: 8192,
            tool_capable: true,
        }],
        fetched_at: 1000,
    };
    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: ModelCacheEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.provider, "test");
    assert_eq!(deserialized.models.len(), 1);
    assert_eq!(deserialized.fetched_at, 1000);
}
