//! Provider-specific LLM generation methods.
//!
//! Each provider has its own `generate_*` method that wraps the shared
//! `generate_completion` base with provider-specific URLs and auth headers.

use super::LlmClient;
use crate::config::ProviderType;
use serde_json::{Value, json};
use std::error::Error;

// Macro for simple OpenAI-compatible providers — defined outside impl block
macro_rules! simple_openai {
    ($fn_name:ident, $config_field:ident, $url:expr) => {
        pub(crate) async fn $fn_name(
            &self,
            messages: &[Value],
            model_override: Option<&str>,
        ) -> Result<Value, Box<dyn Error + Send + Sync>> {
            let api_key = self.config.$config_field.as_deref().unwrap_or("");
            let res_json = self
                .generate_completion(
                    messages,
                    $url,
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
    };
}

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
            ProviderType::Unsloth => self.generate_unsloth(messages, model_override).await,
            ProviderType::AzureOpenAi => self.generate_azure_openai(messages, model_override).await,
            ProviderType::GitHubCopilot => self.generate_github_copilot(messages, model_override).await,
            ProviderType::Bedrock => self.generate_bedrock(messages, model_override).await,
            ProviderType::VertexAi => self.generate_vertex_ai(messages, model_override).await,
            ProviderType::Perplexity => self.generate_perplexity(messages, model_override).await,
            ProviderType::Cohere => self.generate_cohere(messages, model_override).await,
            ProviderType::Cerebras => self.generate_cerebras(messages, model_override).await,
            ProviderType::AlibabaCloud => self.generate_alibaba_cloud(messages, model_override).await,
            ProviderType::VeniceAi => self.generate_venice_ai(messages, model_override).await,
            ProviderType::Nvidia => self.generate_nvidia(messages, model_override).await,
            ProviderType::FireworksAi => self.generate_fireworks_ai(messages, model_override).await,
            ProviderType::SambaNova => self.generate_sambanova(messages, model_override).await,
            ProviderType::OctoAi => self.generate_octo_ai(messages, model_override).await,
            ProviderType::Anyscale => self.generate_anyscale(messages, model_override).await,
            ProviderType::LambdaLabs => self.generate_lambda_labs(messages, model_override).await,
            ProviderType::RunPod => self.generate_runpod(messages, model_override).await,
            ProviderType::Modal => self.generate_modal(messages, model_override).await,
            ProviderType::HuggingFace => self.generate_huggingface(messages, model_override).await,
            ProviderType::LMStudio => self.generate_lmstudio(messages, model_override).await,
            ProviderType::TGI => self.generate_tgi(messages, model_override).await,
            ProviderType::VLLM => self.generate_vllm(messages, model_override).await,
            ProviderType::CustomOpenAi => self.generate_custom_openai(messages, model_override).await,
        }
    }

    // Azure OpenAI
    pub(crate) async fn generate_azure_openai(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let api_key = self.config.azure_openai_key.as_deref().unwrap_or("");
        let endpoint = self
            .config
            .azure_openai_endpoint
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");
        let res_json = self
            .generate_completion(messages, endpoint, Some(("api-key", api_key.to_string())), model_override)
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    // GitHub Copilot
    pub(crate) async fn generate_github_copilot(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let token = self.config.github_copilot_token.as_deref().unwrap_or("");
        let endpoint = "https://api.githubcopilot.com/v1/chat/completions";
        let res_json = self
            .generate_completion(messages, endpoint, Some(("Authorization", format!("Bearer {}", token))), model_override)
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    // Bedrock
    pub(crate) async fn generate_bedrock(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let key = self.config.bedrock_access_key.as_deref().unwrap_or("");
        let endpoint = "https://bedrock.amazonaws.com/v1/chat/completions";
        let res_json = self
            .generate_completion(messages, endpoint, Some(("x-amz-access-key", key.to_string())), model_override)
            .await?;
        Ok(res_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .cloned()
            .unwrap_or(json!({})))
    }

    // Vertex AI
    pub(crate) async fn generate_vertex_ai(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let project = self.config.vertex_ai_project.as_deref().unwrap_or("");
        let endpoint = format!("https://{project}-aiplatform.googleapis.com/v1/projects/{project}/locations/us-central1/publishers/google/models/{model}", model = model_override.unwrap_or("default-model"));
        let res_json = self
            .generate_completion(messages, &endpoint, None, model_override)
            .await?;
        Ok(res_json
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .cloned()
            .unwrap_or(json!({})))
    }

    simple_openai!(generate_perplexity, perplexity_api_key, "https://api.perplexity.ai/v1/chat/completions");
    simple_openai!(generate_cohere, cohere_api_key, "https://api.cohere.com/v1/chat/completions");
    simple_openai!(generate_cerebras, cerebras_api_key, "https://api.cerebras.ai/v1/chat/completions");
    simple_openai!(generate_alibaba_cloud, alibaba_cloud_key, "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions");
    simple_openai!(generate_venice_ai, venice_ai_key, "https://api.venice.ai/api/v1/chat/completions");
    simple_openai!(generate_nvidia, nvidia_api_key, "https://integrate.api.nvidia.com/v1/chat/completions");
    simple_openai!(generate_fireworks_ai, fireworks_ai_key, "https://api.fireworks.ai/inference/v1/chat/completions");
    simple_openai!(generate_sambanova, sambanova_api_key, "https://api.sambanova.ai/v1/chat/completions");
    simple_openai!(generate_octo_ai, octo_ai_key, "https://api.octoai.com/v1/chat/completions");
    simple_openai!(generate_anyscale, anyscale_api_key, "https://api.anyscale.com/v1/chat/completions");
    simple_openai!(generate_lambda_labs, lambda_labs_api_key, "https://api.lambdalabs.com/v1/chat/completions");
    simple_openai!(generate_runpod, runpod_api_key, "https://api.runpod.ai/v1/chat/completions");
    simple_openai!(generate_modal, modal_api_key, "https://api.modal.com/v1/chat/completions");
    simple_openai!(generate_huggingface, huggingface_api_key, "https://api-inference.huggingface.co/v1/chat/completions");
    simple_openai!(generate_lmstudio, lmstudio_url, "http://localhost:1234/v1/chat/completions");
    simple_openai!(generate_tgi, tgi_url, "http://localhost:8080/v1/chat/completions");
    simple_openai!(generate_vllm, vllm_url, "http://localhost:8000/v1/chat/completions");
    simple_openai!(generate_custom_openai, custom_openai_url, "");


    /// Dispatch with automatic fallback: tries primary provider, then each in fallback_chain.
    pub(crate) async fn dispatch_with_fallback(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
        progress_tx: &tokio::sync::mpsc::Sender<String>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Try primary provider first
        let result = self.dispatch_provider(messages, model_override).await;
        if result.is_ok() {
            return result;
        }

        // If fallback chain is empty, return the original error
        if self.config.fallback_chain.is_empty() {
            return result;
        }

        let primary_err = result.unwrap_err();
        let _ = progress_tx
            .send(format!(
                "opencrust: Primary provider failed: {}. Trying fallback...",
                primary_err
            ))
            .await;

        // Try each fallback provider
        for fallback_name in &self.config.fallback_chain {
            let fallback_provider: ProviderType = match fallback_name.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Skip if it's the same as primary
            if fallback_provider == self.config.provider {
                continue;
            }

            // Check if the fallback provider has required API keys
            let has_key = match fallback_provider {
                ProviderType::Ollama => true, // No key needed
                ProviderType::OpenRouter => self.config.openrouter_key.is_some(),
                ProviderType::OpenAI => self.config.openai_key.is_some(),
                ProviderType::Gemini => self.config.gemini_api_key.is_some(),
                ProviderType::Mistral => self.config.mistral_api_key.is_some(),
                ProviderType::Anthropic => self.config.anthropic_api_key.is_some(),
                ProviderType::Groq => self.config.groq_api_key.is_some(),
                ProviderType::TogetherAi => self.config.together_api_key.is_some(),
                ProviderType::Replicate => self.config.replicate_api_key.is_some(),
                ProviderType::DeepSeek => self.config.deepseek_api_key.is_some(),
                ProviderType::LocalAi => self.config.localai_url.is_some(),
                ProviderType::Unsloth => self.config.unsloth_url.is_some(),
                ProviderType::AzureOpenAi => self.config.azure_openai_key.is_some(),
                ProviderType::GitHubCopilot => self.config.github_copilot_token.is_some(),
                ProviderType::Bedrock => self.config.bedrock_access_key.is_some(),
                ProviderType::VertexAi => self.config.vertex_ai_project.is_some(),
                ProviderType::Perplexity => self.config.perplexity_api_key.is_some(),
                ProviderType::Cohere => self.config.cohere_api_key.is_some(),
                ProviderType::Cerebras => self.config.cerebras_api_key.is_some(),
                ProviderType::AlibabaCloud => self.config.alibaba_cloud_key.is_some(),
                ProviderType::VeniceAi => self.config.venice_ai_key.is_some(),
                ProviderType::Nvidia => self.config.nvidia_api_key.is_some(),
                ProviderType::FireworksAi => self.config.fireworks_ai_key.is_some(),
                ProviderType::SambaNova => self.config.sambanova_api_key.is_some(),
                ProviderType::OctoAi => self.config.octo_ai_key.is_some(),
                ProviderType::Anyscale => self.config.anyscale_api_key.is_some(),
                ProviderType::LambdaLabs => self.config.lambda_labs_api_key.is_some(),
                ProviderType::RunPod => self.config.runpod_api_key.is_some(),
                ProviderType::Modal => self.config.modal_api_key.is_some(),
                ProviderType::HuggingFace => self.config.huggingface_api_key.is_some(),
                ProviderType::LMStudio => self.config.lmstudio_url.is_some(),
                ProviderType::TGI => self.config.tgi_url.is_some(),
                ProviderType::VLLM => self.config.vllm_url.is_some(),
                ProviderType::CustomOpenAi => self.config.custom_openai_url.is_some(),
            };

            if !has_key {
                continue;
            }

            let _ = progress_tx
                .send(format!(
                    "opencrust: Trying fallback provider: {}",
                    fallback_name
                ))
                .await;

            // Create a temporary config with the fallback provider
            let mut fallback_config = (*self.config).clone();
            fallback_config.provider = fallback_provider;
            // Don't change the model — use the same model for the fallback

            // Try to generate with fallback config
            // We can't easily swap the config mid-request, so we dispatch by name
            let result = match self.config.fallback_chain.len() {
                0 => unreachable!(),
                _ => {
                    // Use the provider-specific method directly
                    match fallback_name.parse::<ProviderType>() {
                        Ok(ProviderType::Ollama) => {
                            self.generate_ollama(messages, model_override).await
                        }
                        Ok(ProviderType::OpenRouter) => {
                            self.generate_openrouter(messages, model_override).await
                        }
                        Ok(ProviderType::OpenAI) => {
                            self.generate_openai(messages, model_override).await
                        }
                        Ok(ProviderType::Gemini) => {
                            self.generate_gemini(messages, model_override).await
                        }
                        Ok(ProviderType::Mistral) => {
                            self.generate_mistral(messages, model_override).await
                        }
                        Ok(ProviderType::Anthropic) => {
                            self.generate_anthropic(messages, model_override).await
                        }
                        Ok(ProviderType::Groq) => {
                            self.generate_groq(messages, model_override).await
                        }
                        Ok(ProviderType::TogetherAi) => {
                            self.generate_together_ai(messages, model_override).await
                        }
                        Ok(ProviderType::Replicate) => {
                            self.generate_replicate(messages, model_override).await
                        }
                        Ok(ProviderType::DeepSeek) => {
                            self.generate_deepseek(messages, model_override).await
                        }
                        Ok(ProviderType::LocalAi) => {
                            self.generate_local_ai(messages, model_override).await
                        }
                        Ok(ProviderType::Unsloth) => {
                            self.generate_unsloth(messages, model_override).await
                        }
                        Ok(ProviderType::AzureOpenAi) => {
                            self.generate_azure_openai(messages, model_override).await
                        }
                        Ok(ProviderType::GitHubCopilot) => {
                            self.generate_github_copilot(messages, model_override).await
                        }
                        Ok(ProviderType::Bedrock) => {
                            self.generate_bedrock(messages, model_override).await
                        }
                        Ok(ProviderType::VertexAi) => {
                            self.generate_vertex_ai(messages, model_override).await
                        }
                        Ok(ProviderType::Perplexity) => {
                            self.generate_perplexity(messages, model_override).await
                        }
                        Ok(ProviderType::Cohere) => {
                            self.generate_cohere(messages, model_override).await
                        }
                        Ok(ProviderType::Cerebras) => {
                            self.generate_cerebras(messages, model_override).await
                        }
                        Ok(ProviderType::AlibabaCloud) => {
                            self.generate_alibaba_cloud(messages, model_override).await
                        }
                        Ok(ProviderType::VeniceAi) => {
                            self.generate_venice_ai(messages, model_override).await
                        }
                        Ok(ProviderType::Nvidia) => {
                            self.generate_nvidia(messages, model_override).await
                        }
                        Ok(ProviderType::FireworksAi) => {
                            self.generate_fireworks_ai(messages, model_override).await
                        }
                        Ok(ProviderType::SambaNova) => {
                            self.generate_sambanova(messages, model_override).await
                        }
                        Ok(ProviderType::OctoAi) => {
                            self.generate_octo_ai(messages, model_override).await
                        }
                        Ok(ProviderType::Anyscale) => {
                            self.generate_anyscale(messages, model_override).await
                        }
                        Ok(ProviderType::LambdaLabs) => {
                            self.generate_lambda_labs(messages, model_override).await
                        }
                        Ok(ProviderType::RunPod) => {
                            self.generate_runpod(messages, model_override).await
                        }
                        Ok(ProviderType::Modal) => {
                            self.generate_modal(messages, model_override).await
                        }
                        Ok(ProviderType::HuggingFace) => {
                            self.generate_huggingface(messages, model_override).await
                        }
                        Ok(ProviderType::LMStudio) => {
                            self.generate_lmstudio(messages, model_override).await
                        }
                        Ok(ProviderType::TGI) => {
                            self.generate_tgi(messages, model_override).await
                        }
                        Ok(ProviderType::VLLM) => {
                            self.generate_vllm(messages, model_override).await
                        }
                        Ok(ProviderType::CustomOpenAi) => {
                            self.generate_custom_openai(messages, model_override).await
                        }
                        Err(_) => continue,
                    }
                }
            };

            if result.is_ok() {
                return result;
            }
        }

        // All fallbacks exhausted — return original error
        Err(primary_err)
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

    /// Generate via Unsloth Studio (OpenAI-compatible local inference API)
    /// Default URL: http://localhost:8000/v1
    pub(crate) async fn generate_unsloth(
        &self,
        messages: &[Value],
        model_override: Option<&str>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let base_url = self
            .config
            .unsloth_url
            .as_deref()
            .unwrap_or("http://localhost:8000");
        let api_key = self.config.unsloth_api_key.as_deref().unwrap_or("");
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
        | ProviderType::LocalAi
        | ProviderType::Unsloth => res
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
        ProviderType::AzureOpenAi
        | ProviderType::GitHubCopilot
        | ProviderType::Bedrock
        | ProviderType::VertexAi
        | ProviderType::Perplexity
        | ProviderType::Cohere
        | ProviderType::Cerebras
        | ProviderType::AlibabaCloud
        | ProviderType::VeniceAi
        | ProviderType::Nvidia
        | ProviderType::FireworksAi
        | ProviderType::SambaNova
        | ProviderType::OctoAi
        | ProviderType::Anyscale
        | ProviderType::LambdaLabs
        | ProviderType::RunPod
        | ProviderType::Modal
        | ProviderType::HuggingFace
        | ProviderType::LMStudio
        | ProviderType::TGI
        | ProviderType::VLLM
        | ProviderType::CustomOpenAi => res
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string()),
        ProviderType::Anthropic => res
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string()),
    }
}
