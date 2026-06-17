//! Token budget and cost tracking for LLM usage
//!
//! Tracks token usage per session, per agent, and per provider.
//! Provides cost estimation based on provider pricing and implements
//! budget warnings and hard stops.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Token usage statistics for a single LLM request.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    #[expect(dead_code, reason = "public API for detailed usage reporting")]
    pub prompt_tokens: u32,
    #[expect(dead_code, reason = "public API for detailed usage reporting")]
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost: f64,
}

/// Represents a token budget for a session.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    #[expect(dead_code, reason = "public API for session identification")]
    pub session_id: String,
    pub max_tokens: u32,
    pub current_tokens: u32,
    pub total_cost: f64,
    pub warning_threshold: f64,
    pub stop_threshold: f64,
}

impl TokenBudget {
    /// Create a new token budget.
    pub fn new(session_id: String, max_tokens: u32) -> Self {
        Self {
            session_id,
            max_tokens,
            current_tokens: 0,
            total_cost: 0.0,
            warning_threshold: 0.75,
            stop_threshold: 0.90,
        }
    }

    pub fn add_usage(&mut self, usage: TokenUsage) {
        self.current_tokens += usage.total_tokens;
        self.total_cost += usage.cost;
    }

    /// Get usage percentage (0.0 - 1.0).
    pub fn usage_percentage(&self) -> f64 {
        self.current_tokens as f64 / self.max_tokens as f64
    }

    /// Check if at warning threshold (75%).
    pub fn at_warning_threshold(&self) -> bool {
        self.usage_percentage() >= self.warning_threshold
    }

    /// Check if at stop threshold (90%).
    pub fn at_stop_threshold(&self) -> bool {
        self.usage_percentage() >= self.stop_threshold
    }

    /// Get remaining tokens.
    #[expect(dead_code, reason = "public API for budget display")]
    pub fn remaining_tokens(&self) -> u32 {
        self.max_tokens.saturating_sub(self.current_tokens)
    }
}

/// Manages token budgets for multiple sessions.
#[derive(Debug, Clone)]
pub struct TokenBudgetManager {
    budgets: Arc<RwLock<HashMap<String, TokenBudget>>>,
}

impl TokenBudgetManager {
    pub fn new() -> Self {
        Self {
            budgets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new token budget for a session.
    pub async fn create_budget(&self, session_id: String, max_tokens: u32) -> TokenBudget {
        let mut budgets = self.budgets.write().await;
        let budget = TokenBudget::new(session_id.clone(), max_tokens);
        budgets.insert(session_id, budget.clone());
        budget
    }

    /// Get the budget for a session.
    pub async fn get_budget(&self, session_id: &str) -> Option<TokenBudget> {
        let budgets = self.budgets.read().await;
        budgets.get(session_id).cloned()
    }

    /// Record token usage for a session.
    pub async fn add_usage(&self, session_id: &str, usage: TokenUsage) {
        let mut budgets = self.budgets.write().await;
        if let Some(budget) = budgets.get_mut(session_id) {
            budget.add_usage(usage);
        }
    }

    /// Get usage percentage for a session (0.0 - 1.0).
    #[expect(dead_code, reason = "public API for external budget monitoring")]
    pub async fn usage_percentage(&self, session_id: &str) -> f64 {
        if let Some(budget) = self.get_budget(session_id).await {
            budget.usage_percentage()
        } else {
            0.0
        }
    }

    /// Check if a session has exceeded its budget.
    pub async fn is_over_budget(&self, session_id: &str) -> bool {
        if let Some(budget) = self.get_budget(session_id).await {
            budget.at_stop_threshold()
        } else {
            false
        }
    }
}

/// Pricing information for a provider/model.
#[derive(Debug, Clone)]
pub struct ProviderPricing {
    #[expect(dead_code, reason = "public API for pricing identification")]
    pub provider: String,
    #[expect(dead_code, reason = "public API for pricing identification")]
    pub model: String,
    pub prompt_price: f64,
    pub completion_price: f64,
}

impl ProviderPricing {
    pub fn new(provider: &str, model: &str, prompt_price: f64, completion_price: f64) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            prompt_price,
            completion_price,
        }
    }

    /// Estimate cost for a given number of tokens.
    pub fn estimate_cost(&self, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        let prompt_cost = (prompt_tokens as f64 / 1000.0) * self.prompt_price;
        let completion_cost = (completion_tokens as f64 / 1000.0) * self.completion_price;
        prompt_cost + completion_cost
    }
}

/// Returns pricing for known providers/models.
pub fn get_provider_pricing(provider: &str, model: &str) -> ProviderPricing {
    match (provider, model) {
        ("openai", "gpt-4") => ProviderPricing::new(provider, model, 0.03, 0.06),
        ("openai", "gpt-4o") => ProviderPricing::new(provider, model, 0.005, 0.015),
        ("openai", "gpt-4o-mini") => ProviderPricing::new(provider, model, 0.00015, 0.0006),
        ("openai", "gpt-3.5-turbo") => ProviderPricing::new(provider, model, 0.0005, 0.0015),
        ("anthropic", "claude-sonnet-4-20250514") => {
            ProviderPricing::new(provider, model, 0.003, 0.015)
        }
        ("anthropic", "claude-3-5-sonnet-20241022") => {
            ProviderPricing::new(provider, model, 0.003, 0.015)
        }
        ("anthropic", "claude-3-haiku-20240307") => {
            ProviderPricing::new(provider, model, 0.00025, 0.00125)
        }
        ("gemini", "gemini-pro") => ProviderPricing::new(provider, model, 0.0005, 0.0015),
        ("gemini", "gemini-2.0-flash") => ProviderPricing::new(provider, model, 0.0001, 0.0004),
        ("groq", _) => ProviderPricing::new(provider, model, 0.0005, 0.0015),
        ("deepseek", _) => ProviderPricing::new(provider, model, 0.00014, 0.00028),
        ("ollama", _) => ProviderPricing::new(provider, model, 0.0, 0.0),
        _ => ProviderPricing::new(provider, model, 0.0, 0.0),
    }
}

/// Extract token usage from a provider response (best-effort).
pub fn extract_usage_from_response(provider: &str, response: &serde_json::Value) -> TokenUsage {
    // Try standard OpenAI-compatible usage field
    if let Some(usage) = response.get("usage") {
        let prompt_tokens = usage
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let completion_tokens = usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let total_tokens = prompt_tokens + completion_tokens;
        let pricing = get_provider_pricing(provider, "");
        let cost = pricing.estimate_cost(prompt_tokens, completion_tokens);

        return TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
            cost,
        };
    }

    TokenUsage::default()
}
