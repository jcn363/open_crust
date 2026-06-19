//! Context management: system prompt construction and auto-summarization.

use super::LlmClient;
use super::types::BASE_SYSTEM_PROMPT;
use crate::config::ProviderType;
use crate::rules;
use serde_json::{Value, json};

impl LlmClient {
    /// Build the system prompt for the conversation, including rules, skills,
    /// pinned files, and active goal.
    pub(crate) async fn build_system_prompt(&self) -> String {
        let rules_content = rules::load_rules(&self.config.instructions);
        let mut system_prompt = format!(
            "{}\n\n## Instructions and Rules\n{}",
            BASE_SYSTEM_PROMPT, rules_content
        );

        // Skills integration
        {
            let skills = self.skill_manager.lock().await;
            let available_skills = skills.get_available_skills_xml();
            system_prompt.push_str("\n\n## Available Skills\n");
            system_prompt.push_str(&available_skills);
        }

        // Pinned files integration
        {
            let pinned = self.pinned_files.lock().await;
            if !pinned.is_empty() {
                system_prompt.push_str("\n\n## Pinned Context\n");
                for path in pinned.iter() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        system_prompt
                            .push_str(&format!("<file path=\"{}\">\n{}\n</file>\n", path, content));
                    }
                }
            }
        }

        // Goal integration
        if let Some(goal_prompt) = self.get_goal_prompt() {
            system_prompt.push_str(&goal_prompt);
        }

        system_prompt
    }

    /// Auto-summarize context when approaching budget threshold.
    /// Returns (should_summarize, summary_message).
    pub(crate) async fn check_and_summarize_context(
        &self,
        messages_history: &mut Vec<Value>,
    ) -> (bool, Option<String>) {
        let total_chars: usize = messages_history
            .iter()
            .map(|m| {
                m.get("content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0)
            })
            .sum();
        let estimated_tokens = (total_chars / 4) as u64;

        let context_limit = self.config.context_limit();
        let threshold = (context_limit as f64 * self.config.summarization_threshold()) as u64;

        if estimated_tokens < threshold {
            return (false, None);
        }

        let system_prompt = if let Some(first) = messages_history.first() {
            if first.get("role").and_then(|r| r.as_str()) == Some("system") {
                Some(first.clone())
            } else {
                None
            }
        } else {
            None
        };

        if messages_history.len() <= 11 {
            return (false, None);
        }

        let split_point = messages_history.len() - 10;
        let old_messages: Vec<Value> = messages_history.drain(..split_point).collect();

        let messages_to_summarize: Vec<String> = old_messages
            .iter()
            .filter_map(|m| {
                let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
                let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if !content.is_empty() {
                    Some(format!("{}: {}", role, content))
                } else {
                    None
                }
            })
            .collect();

        let summarize_prompt = format!(
            "Please provide a concise summary of the following conversation history, preserving key technical details, decisions, and context:\n\n{}",
            messages_to_summarize.join("\n")
        );

        let messages = [json!({"role": "user", "content": &summarize_prompt})];
        let summary_result = match self.config.provider {
            ProviderType::Ollama => self.generate_ollama(&messages, None).await,
            ProviderType::OpenRouter => self.generate_openrouter(&messages, None).await,
            ProviderType::OpenAI => self.generate_openai(&messages, None).await,
            ProviderType::Gemini => self.generate_gemini(&messages, None).await,
            ProviderType::Mistral => self.generate_mistral(&messages, None).await,
            ProviderType::Anthropic => self.generate_anthropic(&messages, None).await,
            ProviderType::Groq
            | ProviderType::TogetherAi
            | ProviderType::Replicate
            | ProviderType::DeepSeek
            | ProviderType::LocalAi
            | ProviderType::Unsloth
            | ProviderType::AzureOpenAi
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
            | ProviderType::CustomOpenAi => self.generate_openai(&messages, None).await,
        };

        let summary = match summary_result {
            Ok(mut res) => res
                .get_mut("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            Err(_) => "Previous conversation context (summarized due to length)".to_string(),
        };

        let mut new_history = Vec::new();
        if let Some(sp) = system_prompt {
            new_history.push(sp);
        }
        new_history.push(json!({
            "role": "system",
            "content": format!("[Previous conversation summary: {}]", summary)
        }));
        new_history.append(messages_history);

        *messages_history = new_history;

        (true, Some(summary))
    }
}
