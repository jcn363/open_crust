//! Simple query and input completion methods (no tool execution).

use super::LlmClient;
use serde_json::json;
use std::error::Error;

impl LlmClient {
    /// Single-shot query without tool execution — for multi-agent comparison.
    pub async fn query_simple(
        &self,
        prompt: &str,
        model_override: Option<&str>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let messages = vec![json!({"role": "user", "content": prompt})];

        let (provider, model) = if let Some(override_str) = model_override {
            self.config.parse_model_string(override_str)
        } else {
            (self.config.provider.clone(), self.config.model.clone())
        };

        let res = self.dispatch_provider(&messages, Some(&model)).await?;

        crate::llm::providers::extract_content(&provider, &res)
            .ok_or_else(|| "No content in response".into())
    }

    /// Generate lightweight input completion (ghost text).
    pub async fn generate_input_completion(
        &self,
        current_input: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        if current_input.trim().is_empty() {
            return Ok(String::new());
        }

        let truncated = if current_input.len() > 200 {
            &current_input[current_input.len() - 200..]
        } else {
            current_input
        };

        let prompt = format!(
            "Complete the following input with a short, relevant continuation (max 50 chars). Only return the continuation, no quotes, no explanation:\n\n{}",
            truncated
        );

        let messages = vec![json!({"role": "user", "content": &prompt})];
        let result = self.dispatch_provider(&messages, None).await?;

        let content = crate::llm::providers::extract_content(&self.config.provider, &result)
            .unwrap_or_default();

        Ok(content.to_string())
    }
}
