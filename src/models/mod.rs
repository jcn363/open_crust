//! LLM provider model management and caching
//!
//! Fetches available models from providers (OpenRouter, Ollama, etc.),
//! caches them for fast access, and provides bundled defaults as fallback.
//! Supports automatic periodic refresh for up-to-date model lists.

use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A model entry provided by a provider API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub context_length: u64,
    #[serde(default = "default_tool_capable")]
    pub tool_capable: bool,
}

fn default_tool_capable() -> bool {
    true
}

/// Cached response from a provider API listing models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCacheEntry {
    pub provider: String,
    pub models: Vec<ProviderModel>,
    pub fetched_at: u64, // unix timestamp
}

/// Manages the local persistent cache of provider model lists
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a new CacheManager storing files under `~/.cache/opencrust/models/`
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencrust")
            .join("models");
        Self { cache_dir }
    }

    /// Load models for a given provider from cache, if not stale
    pub fn load(&self, provider: &str, max_age: Duration) -> Option<Vec<ProviderModel>> {
        let path = self.cache_dir.join(format!("{}.json", provider));
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        let entry: ModelCacheEntry = serde_json::from_str(&content).ok()?;

        // Check staleness
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age = Duration::from_secs(now.saturating_sub(entry.fetched_at));
        if age > max_age {
            return None; // stale
        }
        Some(entry.models)
    }

    /// Save models for a provider to cache
    pub fn save(&self, provider: &str, models: Vec<ProviderModel>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = ModelCacheEntry {
            provider: provider.to_string(),
            models,
            fetched_at: now,
        };
        if let Ok(content) = serde_json::to_string_pretty(&entry) {
            if let Err(e) = std::fs::create_dir_all(&self.cache_dir) {
                eprintln!("Warning: Failed to create model cache dir: {}", e);
            }
            let path = self.cache_dir.join(format!("{}.json", provider));
            if let Err(e) = std::fs::write(&path, &content) {
                eprintln!("Warning: Failed to write model cache: {}", e);
            }
        }
    }

    /// Remove deprecated models from cache that are no longer in the fresh list
    #[expect(dead_code, reason = "cache synchronization API")]
    pub fn sync(&self, provider: &str, fresh_models: &[ProviderModel]) -> Vec<ProviderModel> {
        let path = self.cache_dir.join(format!("{}.json", provider));
        if !path.exists() {
            return fresh_models.to_vec();
        }
        let content = std::fs::read_to_string(&path).ok();
        let current = content
            .and_then(|c| serde_json::from_str::<ModelCacheEntry>(&c).ok())
            .map(|e| e.models)
            .unwrap_or_default();

        // Only keep models that appear in the fresh list
        let fresh_ids: std::collections::HashSet<String> =
            fresh_models.iter().map(|m| m.id.clone()).collect();
        current
            .into_iter()
            .filter(|m| fresh_ids.contains(&m.id))
            .collect()
    }

    /// Get the last update timestamp for a provider
    #[expect(dead_code, reason = "cache staleness check API")]
    pub fn last_updated(&self, provider: &str) -> Option<u64> {
        let path = self.cache_dir.join(format!("{}.json", provider));
        let content = std::fs::read_to_string(&path).ok()?;
        let entry: ModelCacheEntry = serde_json::from_str(&content).ok()?;
        Some(entry.fetched_at)
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Fetch live model lists from provider APIs
pub struct ModelFetcher {
    client: reqwest::Client,
    cache: CacheManager,
}

impl ModelFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: CacheManager::new(),
        }
    }

    /// Fetch models from a provider's API.
    /// Returns the list of models, or a cached/stale list on failure.
    ///
    /// Supported providers:
    /// - openai: <https://api.openai.com/v1/models>
    /// - anthropic: <https://api.anthropic.com/v1/models> (API key required)
    /// - openrouter: <https://openrouter.ai/api/v1/models>
    /// - ollama: http://localhost:11434/api/tags
    ///
    /// For providers that require an API key, pass it via `api_key`.
    pub async fn fetch(
        &self,
        provider: &str,
        api_key: Option<&str>,
        base_url: Option<&str>,
    ) -> Vec<ProviderModel> {
        let url = self.build_url(provider, base_url);

        // Try API call
        let mut req = self.client.get(&url);
        if let Some(key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                match self.parse_response(provider, resp).await {
                    Ok(models) => {
                        // Cache fresh models, removing deprecated ones
                        self.cache.save(provider, models.clone());
                        return models;
                    }
                    Err(_) => {
                        // Fall back to cache
                    }
                }
            }
            _ => {
                // Network error or non-success response
            }
        }

        // Fallback to cache (accept stale data)
        self.cache
            .load(provider, Duration::from_secs(7 * 86400))
            .unwrap_or_default()
    }

    /// Refresh models in the background and return updated list.
    #[expect(dead_code, reason = "explicit model refresh API")]
    pub async fn refresh(
        &self,
        provider: &str,
        api_key: Option<&str>,
        base_url: Option<&str>,
    ) -> Vec<ProviderModel> {
        self.fetch(provider, api_key, base_url).await
    }

    fn build_url(&self, provider: &str, base_url: Option<&str>) -> String {
        match provider {
            "openai" => "https://api.openai.com/v1/models".to_string(),
            "openrouter" => "https://openrouter.ai/api/v1/models".to_string(),
            "anthropic" => {
                base_url.unwrap_or("https://api.anthropic.com").to_string() + "/v1/models"
            }
            "ollama" => {
                let ollama_url = base_url.unwrap_or("http://localhost:11434");
                format!("{}/api/tags", ollama_url)
            }
            "gemini" => "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            "mistral" => "https://api.mistral.ai/v1/models".to_string(),
            "groq" => "https://api.groq.com/openai/v1/models".to_string(),
            "togetherai" => "https://api.together.xyz/v1/models".to_string(),
            "deepseek" => "https://api.deepseek.com/v1/models".to_string(),
            _ => format!("https://api.{}.com/v1/models", provider),
        }
    }

    async fn parse_response(
        &self,
        provider: &str,
        resp: reqwest::Response,
    ) -> Result<Vec<ProviderModel>, Box<dyn std::error::Error + Send + Sync>> {
        let body: serde_json::Value = resp.json().await?;

        let models = match provider {
            "openai" | "openrouter" | "mistral" | "groq" | "togetherai" | "deepseek" => {
                // OpenAI-compatible: { data: [{ id, ... }] }
                body.get("data")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| {
                                let id = item.get("id").and_then(|i| i.as_str())?;
                                Some(ProviderModel {
                                    id: id.to_string(),
                                    name: item
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or(id)
                                        .to_string(),
                                    provider: provider.to_string(),
                                    description: item
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    context_length: item
                                        .get("context_length")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(4096),
                                    tool_capable: item
                                        .get("tool_capable")
                                        .and_then(|c| c.as_bool())
                                        .unwrap_or(true),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }
            "anthropic" => {
                // Anthropic: { data: [{ id, ... }] }
                body.get("data")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| {
                                let id = item.get("id").and_then(|i| i.as_str())?;
                                Some(ProviderModel {
                                    id: id.to_string(),
                                    name: item
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or(id)
                                        .to_string(),
                                    provider: provider.to_string(),
                                    description: item
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    context_length: item
                                        .get("context_length")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(100_000),
                                    tool_capable: true,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }
            "ollama" => {
                // Ollama: { models: [{ name, ... }] }
                body.get("models")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| {
                                let name = item.get("name").and_then(|n| n.as_str())?;
                                Some(ProviderModel {
                                    id: name.to_string(),
                                    name: name.to_string(),
                                    provider: provider.to_string(),
                                    description: String::new(),
                                    context_length: item
                                        .get("context_length")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(4096),
                                    tool_capable: true,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }
            "gemini" => {
                // Gemini: { models: [{ name, ... }] }
                body.get("models")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| {
                                let name = item.get("name").and_then(|n| n.as_str())?;
                                let id = name.strip_prefix("models/").unwrap_or(name).to_string();
                                Some(ProviderModel {
                                    id,
                                    name: item
                                        .get("displayName")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or(name)
                                        .to_string(),
                                    provider: provider.to_string(),
                                    description: item
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    context_length: item
                                        .get("inputTokenLimit")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(128_000),
                                    tool_capable: true,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        };

        Ok(models)
    }
}

impl Default for ModelFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// A bundled default model list used when no cache exists
#[cfg(test)]
pub fn bundled_default_models() -> HashMap<String, Vec<ProviderModel>> {
    let mut map = HashMap::new();

    map.insert(
        "openai".to_string(),
        vec![
            ProviderModel {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                provider: "openai".to_string(),
                description: "OpenAI's flagship multimodal model".to_string(),
                context_length: 128_000,
                tool_capable: true,
            },
            ProviderModel {
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4o Mini".to_string(),
                provider: "openai".to_string(),
                description: "Smaller, faster, cheaper version of GPT-4o".to_string(),
                context_length: 128_000,
                tool_capable: true,
            },
        ],
    );

    map.insert(
        "anthropic".to_string(),
        vec![
            ProviderModel {
                id: "claude-sonnet-4-20250514".to_string(),
                name: "Claude Sonnet 4".to_string(),
                provider: "anthropic".to_string(),
                description: "Anthropic's balanced intelligence model".to_string(),
                context_length: 200_000,
                tool_capable: true,
            },
            ProviderModel {
                id: "claude-haiku-3-5-20241022".to_string(),
                name: "Claude Haiku 3.5".to_string(),
                provider: "anthropic".to_string(),
                description: "Anthropic's fastest, most compact model".to_string(),
                context_length: 200_000,
                tool_capable: true,
            },
        ],
    );

    map.insert(
        "openrouter".to_string(),
        vec![ProviderModel {
            id: "openrouter/free".to_string(),
            name: "Free GPT-4o Mini".to_string(),
            provider: "openrouter".to_string(),
            description: "Free model via OpenRouter (no API key required)".to_string(),
            context_length: 128_000,
            tool_capable: true,
        }],
    );

    map
}

#[cfg(test)]
mod tests;
