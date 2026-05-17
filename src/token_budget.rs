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
#[expect(dead_code, reason = "public API for future token cost tracking")]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost: f64,
}

/// Represents a token budget for a session.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    #[expect(dead_code, reason = "exposed for UI display and session tracking")]
    pub session_id: String,
    pub max_tokens: u32,
    pub current_tokens: u32,
    pub total_cost: f64,
    pub warning_threshold: f64,
    pub stop_threshold: f64,
}

#[expect(dead_code, reason = "public API for future token budget enforcement")]
impl TokenBudget {
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

    pub fn usage_percentage(&self) -> f64 {
        self.current_tokens as f64 / self.max_tokens as f64
    }

    pub fn at_warning_threshold(&self) -> bool {
        self.usage_percentage() >= self.warning_threshold
    }

    pub fn at_stop_threshold(&self) -> bool {
        self.usage_percentage() >= self.stop_threshold
    }

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

    #[expect(dead_code, reason = "public API for future budget creation")]
    pub async fn create_budget(&self, session_id: String, max_tokens: u32) -> TokenBudget {
        let mut budgets = self.budgets.write().await;
        let budget = TokenBudget::new(session_id.clone(), max_tokens);
        budgets.insert(session_id, budget.clone());
        budget
    }

    pub async fn get_budget(&self, session_id: &str) -> Option<TokenBudget> {
        let budgets = self.budgets.read().await;
        budgets.get(session_id).cloned()
    }

    #[expect(dead_code, reason = "public API for future usage tracking")]
    pub async fn add_usage(&self, session_id: &str, usage: TokenUsage) {
        let mut budgets = self.budgets.write().await;
        if let Some(budget) = budgets.get_mut(session_id) {
            budget.add_usage(usage);
        }
    }

    #[expect(dead_code, reason = "public API for future usage monitoring")]
    pub async fn usage_percentage(&self, session_id: &str) -> f64 {
        if let Some(budget) = self.get_budget(session_id).await {
            budget.usage_percentage()
        } else {
            0.0
        }
    }
}

/// Pricing information for a provider/model.
#[derive(Debug, Clone)]
pub struct ProviderPricing {
    #[expect(dead_code, reason = "identifier for pricing lookup")]
    pub provider: String,
    #[expect(dead_code, reason = "identifier for pricing lookup")]
    pub model: String,
    pub prompt_price: f64,
    pub completion_price: f64,
}

#[expect(dead_code, reason = "public API for future token cost estimation")]
impl ProviderPricing {
    pub fn new(provider: &str, model: &str, prompt_price: f64, completion_price: f64) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            prompt_price,
            completion_price,
        }
    }

    pub fn estimate_cost(&self, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        let prompt_cost = (prompt_tokens as f64 / 1000.0) * self.prompt_price;
        let completion_cost = (completion_tokens as f64 / 1000.0) * self.completion_price;
        prompt_cost + completion_cost
    }
}

/// Returns pricing for known providers/models.
#[expect(dead_code, reason = "public API for future token cost estimation")]
pub fn get_provider_pricing(provider: &str, model: &str) -> ProviderPricing {
    match (provider, model) {
        ("openai", "gpt-4") => ProviderPricing::new(provider, model, 0.03, 0.06),
        ("openai", "gpt-3.5-turbo") => ProviderPricing::new(provider, model, 0.002, 0.002),
        ("anthropic", "claude-2") => ProviderPricing::new(provider, model, 0.008, 0.024),
        ("gemini", "gemini-pro") => ProviderPricing::new(provider, model, 0.0005, 0.0015),
        ("ollama", _) => ProviderPricing::new(provider, model, 0.0, 0.0),
        _ => ProviderPricing::new(provider, model, 0.0, 0.0),
    }
}
