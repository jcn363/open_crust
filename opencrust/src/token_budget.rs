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
    #[allow(dead_code)]
    pub prompt_tokens: u32,
    #[allow(dead_code)]
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost: f64,
}

/// Represents a token budget for a session.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
}

impl Default for TokenBudgetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenBudgetManager {
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
    #[allow(dead_code)]
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
    pub provider: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_budget_new_sets_correct_defaults() {
        let budget = TokenBudget::new("test-session".to_string(), 1000);
        assert_eq!(budget.session_id, "test-session");
        assert_eq!(budget.max_tokens, 1000);
        assert_eq!(budget.current_tokens, 0);
        assert_eq!(budget.total_cost, 0.0);
        assert!((budget.warning_threshold - 0.75).abs() < f64::EPSILON);
        assert!((budget.stop_threshold - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn token_budget_add_usage_increments_tokens() {
        let mut budget = TokenBudget::new("test".to_string(), 1000);
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 200,
            total_tokens: 300,
            cost: 0.05,
        };
        budget.add_usage(usage);
        assert_eq!(budget.current_tokens, 300);
        assert!((budget.total_cost - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn token_budget_usage_percentage_calculation() {
        let mut budget = TokenBudget::new("test".to_string(), 1000);
        assert!((budget.usage_percentage()).abs() < f64::EPSILON);
        budget.current_tokens = 500;
        assert!((budget.usage_percentage() - 0.5).abs() < f64::EPSILON);
        budget.current_tokens = 1000;
        assert!((budget.usage_percentage() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn token_budget_warning_threshold_at_75_percent() {
        let mut budget = TokenBudget::new("test".to_string(), 1000);
        assert!(!budget.at_warning_threshold());
        budget.current_tokens = 749;
        assert!(!budget.at_warning_threshold());
        budget.current_tokens = 750;
        assert!(budget.at_warning_threshold());
    }

    #[test]
    fn token_budget_stop_threshold_at_90_percent() {
        let mut budget = TokenBudget::new("test".to_string(), 1000);
        assert!(!budget.at_stop_threshold());
        budget.current_tokens = 899;
        assert!(!budget.at_stop_threshold());
        budget.current_tokens = 900;
        assert!(budget.at_stop_threshold());
    }

    #[test]
    fn token_budget_remaining_tokens_saturates_at_zero() {
        let mut budget = TokenBudget::new("test".to_string(), 100);
        assert_eq!(budget.remaining_tokens(), 100);
        budget.current_tokens = 50;
        assert_eq!(budget.remaining_tokens(), 50);
        budget.current_tokens = 150;
        assert_eq!(budget.remaining_tokens(), 0);
    }

    #[tokio::test]
    async fn budget_manager_creates_and_retrieves() {
        let manager = TokenBudgetManager::new();
        manager.create_budget("session-1".to_string(), 5000).await;
        let budget = manager.get_budget("session-1").await;
        assert!(budget.is_some());
        let budget = budget.unwrap();
        assert_eq!(budget.max_tokens, 5000);
        assert_eq!(budget.current_tokens, 0);
    }

    #[tokio::test]
    async fn budget_manager_returns_none_for_unknown() {
        let manager = TokenBudgetManager::new();
        let budget = manager.get_budget("nonexistent").await;
        assert!(budget.is_none());
    }

    #[tokio::test]
    async fn budget_manager_adds_usage() {
        let manager = TokenBudgetManager::new();
        manager.create_budget("session-1".to_string(), 5000).await;
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 200,
            total_tokens: 300,
            cost: 0.01,
        };
        manager.add_usage("session-1", usage).await;
        let budget = manager.get_budget("session-1").await.unwrap();
        assert_eq!(budget.current_tokens, 300);
    }

    #[tokio::test]
    async fn budget_manager_detects_over_budget() {
        let manager = TokenBudgetManager::new();
        manager.create_budget("session-1".to_string(), 100).await;
        let usage = TokenUsage {
            prompt_tokens: 40,
            completion_tokens: 40,
            total_tokens: 80,
            cost: 0.0,
        };
        manager.add_usage("session-1", usage).await;
        assert!(!manager.is_over_budget("session-1").await);
        let usage = TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 10,
            total_tokens: 20,
            cost: 0.0,
        };
        manager.add_usage("session-1", usage).await;
        assert!(manager.is_over_budget("session-1").await);
    }

    #[tokio::test]
    async fn budget_manager_usage_percentage() {
        let manager = TokenBudgetManager::new();
        manager.create_budget("session-1".to_string(), 1000).await;
        assert!((manager.usage_percentage("session-1").await - 0.0).abs() < f64::EPSILON);
        let usage = TokenUsage {
            prompt_tokens: 250,
            completion_tokens: 250,
            total_tokens: 500,
            cost: 0.0,
        };
        manager.add_usage("session-1", usage).await;
        assert!((manager.usage_percentage("session-1").await - 0.5).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn budget_manager_unknown_session_returns_zero() {
        let manager = TokenBudgetManager::new();
        assert!((manager.usage_percentage("nonexistent").await - 0.0).abs() < f64::EPSILON);
        assert!(!manager.is_over_budget("nonexistent").await);
    }

    #[test]
    fn provider_pricing_estimates_cost() {
        let pricing = ProviderPricing::new("test", "model", 0.03, 0.06);
        let cost = pricing.estimate_cost(1000, 500);
        // prompt: 1000/1000 * 0.03 = 0.03, completion: 500/1000 * 0.06 = 0.03
        assert!((cost - 0.06).abs() < f64::EPSILON);
    }

    #[test]
    fn provider_pricing_zero_tokens() {
        let pricing = ProviderPricing::new("test", "model", 0.03, 0.06);
        let cost = pricing.estimate_cost(0, 0);
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn get_provider_pricing_openai_gpt4() {
        let pricing = get_provider_pricing("openai", "gpt-4");
        assert!((pricing.prompt_price - 0.03).abs() < f64::EPSILON);
        assert!((pricing.completion_price - 0.06).abs() < f64::EPSILON);
    }

    #[test]
    fn get_provider_pricing_unknown_returns_zero() {
        let pricing = get_provider_pricing("unknown", "unknown");
        assert!((pricing.prompt_price - 0.0).abs() < f64::EPSILON);
        assert!((pricing.completion_price - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn get_provider_pricing_ollama_is_free() {
        let pricing = get_provider_pricing("ollama", "llama3");
        assert!((pricing.prompt_price - 0.0).abs() < f64::EPSILON);
        assert!((pricing.completion_price - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn extract_usage_from_response_openai_format() {
        let response = serde_json::json!({
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            }
        });
        let usage = extract_usage_from_response("openai", &response);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn extract_usage_from_response_missing_usage() {
        let response = serde_json::json!({});
        let usage = extract_usage_from_response("openai", &response);
        assert_eq!(usage.total_tokens, 0);
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
    }

    #[test]
    fn extract_usage_from_response_partial_usage() {
        let response = serde_json::json!({
            "usage": {
                "prompt_tokens": 100
            }
        });
        let usage = extract_usage_from_response("openai", &response);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 100);
    }
}
