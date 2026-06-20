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
        let mut builder = ContextBuilder::new(MAX_CONTEXT_TOKENS);

        // System prompt + rules as first item
        let system_and_rules = format!(
            "{}\n\n## Instructions and Rules\n{}",
            BASE_SYSTEM_PROMPT, rules_content
        );
        builder.push(system_and_rules);

        // Skills integration as second item
        {
            let skills = self.skill_manager.lock().await;
            let available_skills = skills.get_available_skills_xml();
            builder.push(format!("## Available Skills\n{}", available_skills));
        }

        // Pinned files integration as individual items
        {
            let pinned = self.pinned_files.lock().await;
            if !pinned.is_empty() {
                for path in pinned.iter() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        builder.push(format!("<file path=\"{}\">\n{}\n</file>", path, content));
                    }
                }
            }
        }

        // Goal integration if present
        if let Some(goal_prompt) = self.get_goal_prompt() {
            builder.push(goal_prompt);
        }

        builder.build()
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

        // Truncate summary to MAX_CONTEXT_ITEM_TOKENS if needed
        let summary_tokens = estimate_tokens(&summary);
        let summary = if summary_tokens > MAX_CONTEXT_ITEM_TOKENS {
            // Truncate by character proportion to fit token budget
            let char_budget = (summary.len() * MAX_CONTEXT_ITEM_TOKENS) / summary_tokens;
            let truncated: String = summary.chars().take(char_budget).collect();
            format!("{}... [truncated]", truncated)
        } else {
            summary
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

// ============================================================================
// Context Size Bounds
// ============================================================================

/// Maximum tokens for a single context item
pub const MAX_CONTEXT_ITEM_TOKENS: usize = 10_000;

/// Maximum total context tokens
pub const MAX_CONTEXT_TOKENS: usize = 128_000;

/// Trait for context items that can be bounded
pub trait ContextualItem {
    /// Estimate token count for this item
    fn estimate_tokens(&self) -> usize;

    /// Truncate this item to fit within token budget
    fn truncate_to(&self, max_tokens: usize) -> Self;
}

/// Context builder that enforces size bounds
pub struct ContextBuilder {
    items: Vec<String>,
    total_tokens: usize,
    max_tokens: usize,
}

impl ContextBuilder {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            items: Vec::new(),
            total_tokens: 0,
            max_tokens,
        }
    }

    /// Add a context item, evicting oldest if over budget
    pub fn push(&mut self, item: String) {
        let estimated = estimate_tokens(&item);
        self.items.push(item);
        self.total_tokens += estimated;

        // Evict from front if over budget
        while self.total_tokens > self.max_tokens && self.items.len() > 1 {
            let removed = self.items.remove(0);
            self.total_tokens -= estimate_tokens(&removed);
        }
    }

    /// Get the final context string
    pub fn build(&self) -> String {
        self.items.join("\n\n")
    }

    /// Current token count
    pub fn token_count(&self) -> usize {
        self.total_tokens
    }

    /// Number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

/// Rough token estimate (words / 0.75)
pub fn estimate_tokens(text: &str) -> usize {
    (text.split_whitespace().count() as f32 / 0.75) as usize
}

/// Estimate total tokens across all messages in a conversation history.
pub(crate) fn estimate_message_tokens(messages: &[Value]) -> usize {
    messages
        .iter()
        .filter_map(|m| m.get("content").and_then(|c| c.as_str()))
        .map(estimate_tokens)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("hello world") > 0);
    }

    #[test]
    fn test_context_builder_eviction() {
        let mut builder = ContextBuilder::new(100);
        // Add many items to trigger eviction
        for i in 0..20 {
            builder.push(format!("Item {} with some text content here", i));
        }
        assert!(builder.token_count() <= 100);
    }

    #[test]
    fn test_estimate_message_tokens() {
        let messages = vec![
            json!({"role": "system", "content": "You are a helpful assistant"}),
            json!({"role": "user", "content": "Hello world"}),
            json!({"role": "assistant", "content": "Hi there! How can I help?"}),
        ];
        let tokens = estimate_message_tokens(&messages);
        assert!(tokens > 0);
        // Rough check: "You are a helpful assistant" ~ 5 words / 0.75 = ~7
        // "Hello world" ~ 2 words / 0.75 = ~3
        // "Hi there! How can I help?" ~ 6 words / 0.75 = ~8
        // Total ~ 18
        assert!(tokens >= 15 && tokens <= 25);
    }

    #[test]
    fn test_context_builder_max_tokens() {
        let mut builder = ContextBuilder::new(50);
        builder.push("First item with some content".to_string());
        builder.push("Second item with more content here".to_string());
        builder.push("Third item".to_string());
        // Total should not exceed max_tokens (50)
        assert!(builder.token_count() <= 50);
        // Should have at least 1 item (the most recent)
        assert!(builder.item_count() >= 1);
    }
}
