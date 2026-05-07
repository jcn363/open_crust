use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UsageStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
}

impl UsageStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total tokens used (input + output)
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn add_usage(&mut self, model: &str, input: u64, output: u64) {
        self.input_tokens += input;
        self.output_tokens += output;

        // Very rough cost calculation (placeholder prices per 1M tokens)
        let (in_price, out_price) = match model {
            m if m.contains("gpt-4o") => (5.0, 15.0),
            m if m.contains("claude-3-5-sonnet") => (3.0, 15.0),
            _ => (1.0, 3.0), // Default cheap model
        };

        self.total_cost += (input as f64 / 1_000_000.0) * in_price;
        self.total_cost += (output as f64 / 1_000_000.0) * out_price;
    }
}
