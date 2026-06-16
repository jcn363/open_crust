//! Provider-specific LLM generation methods.
//!
//! Each provider has its own `generate_*` method that wraps the shared
//! `generate_completion` base with provider-specific URLs and auth headers.

use super::LlmClient;
use crate::config::ProviderType;
use serde_json::{Value, json};
use std::error::Error;

impl LlmClient {
    /// Dispatch to the appropriate provider based on config.
    pub(crate) async fn dispatch_provider(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        match self.config.provider {
            ProviderType::Ollama => self.generate_ollama(messages, model_override).await,
            ProviderType::OpenRouter => self.generate_openrouter(messages, model_override).await,
            ProviderType::OpenAI => self.generate_openai(messages, model_override).await,
            ProviderType::Gemini => self.generate_gemini(messages, model_override).await,
            ProviderType::Mistral => self.generate_mistral(messages, model_override).await,
            ProviderType::Anthropic => self.generate_anthropic(messages, model_override).await,
            ProviderType::Groq => self.generate_groq(messages, model_override).await,
            ProviderType::TogetherAi => self.generate_together_ai(messages, model_override).await,
            ProviderType::Replicate => self.generate_replicate(messages, model_override).await,
            ProviderType::DeepSeek => self.generate_deepseek(messages, model_override).await,
            ProviderType::LocalAi => self.generate_local_ai(messages, model_override).await,
        }
    }

    /// Shared base: POST JSON to a provider endpoint, optionally with auth header.
    pub(crate) async fn generate_completion(
        &self,
        messages: &[Value],
        url: &str,
        auth_header: Option<(&str, String)>,
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let model = model_override.unwrap_or(&self.config.model);
        let tools_schema = crate::tool_executor::get_all_tool_schemas(
            &self.mcp_manager,
            &self.custom_tool_manager,
            &self.plugin_manager,
        )
        .await;

        let body = json!({
            "model": model,
            "messages": messages,
            "tools": tools_schema,
            "stream": false
        });

        let mut request = self.client.post(url).json(&body);
        if let Some((header_name, header_value)) = auth_header {
            request = request.header(header_name, header_value);
        }
        let res = request.send().await?;
        let res_json: Value = res.json().await?;
        Ok(res_json)
    }

    pub(crate) async fn generate_ollama(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let ollama_url = self
            .config
            .ollama_url
            .as_deref()
            .unwrap_or("http://localhost:11434");
        self.generate_completion(
            messages,
            &format!("{}/api/chat", ollama_url),
            None,
            model_override,
        )
        .await
    }

    pub(crate) async fn generate_openrouter(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.openrouter_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://openrouter.ai/api/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_openai(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.openai_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.openai.com/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_gemini(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.gemini_api_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_mistral(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.mistral_api_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.mistral.ai/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_anthropic(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.anthropic_api_key.as_deref().unwrap_or("");

        let mut anthropic_messages: Vec<Value> = Vec::new();
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str());
            let content = msg.get("content").and_then(|c| c.as_str());
            if let (Some(r), Some(c)) = (role, content)
                && r != "system"
            {
                anthropic_messages.push(json!({"role": r, "content": c}));
            }
        }

        let model = model_override.unwrap_or(&self.config.model);
        let body = json!({
            "model": model,
            "messages": anthropic_messages,
            "max_tokens": 4096
        });

        let res = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let res_json: Value = res.json().await?;
        Ok(res_json
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_groq(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.groq_api_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.groq.com/openai/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_together_ai(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.together_api_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.together.xyz/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_replicate(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.replicate_api_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.replicate.com/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_deepseek(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.deepseek_api_key.as_deref().unwrap_or("");
        let res_json = self
            .generate_completion(
                messages,
                "https://api.deepseek.com/v1/chat/completions",
                Some(("Authorization", format!("Bearer {}", api_key))),
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    pub(crate) async fn generate_local_ai(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let base_url = self
            .config
            .localai_url
            .as_deref()
            .unwrap_or("http://localhost:8080");
        let api_key = self.config.localai_api_key.as_deref().unwrap_or("");
        let auth = if api_key.is_empty() {
            None
        } else {
            Some(("Authorization", format!("Bearer {}", api_key)))
        };
        let res_json = self
            .generate_completion(
                messages,
                &format!("{}/v1/chat/completions", base_url),
                auth,
                model_override,
            )
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }
}

/// Extract content from a provider response, handling multiple response formats.
pub(crate) fn extract_content(provider: &ProviderType, res: &Value) -> Option<String> {
    match provider {
        ProviderType::Ollama => res
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string()),
        ProviderType::OpenRouter
        | ProviderType::OpenAI
        | ProviderType::Mistral
        | ProviderType::Groq
        | ProviderType::TogetherAi
        | ProviderType::Replicate
        | ProviderType::DeepSeek
        | ProviderType::LocalAi => res
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string()),
        ProviderType::Gemini => res
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string()),
        ProviderType::Anthropic => res
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string()),
    }
}
