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

        let (in_price, out_price) = match model {
            m if m.contains("gpt-4o") => (2.50, 10.0),
            m if m.contains("gpt-4o-mini") => (0.15, 0.60),
            m if m.contains("claude-3-5-sonnet") | m.contains("claude-sonnet-4") => (3.0, 15.0),
            m if m.contains("claude-haiku") => (0.80, 4.0),
            m if m.contains("claude-opus") => (15.0, 75.0),
            m if m.contains("gemini") => (0.10, 0.40),
            m if m.contains("mistral") | m.contains("mixtral") => (0.15, 0.60),
            m if m.contains("deepseek") => (0.14, 0.28),
            m if m.contains("command-r") => (0.15, 0.60),
            m if m.contains("llama") | m.contains("llama3") | m.contains("qwen") => (0.50, 1.50),
            _ => (1.0, 3.0),
        };

        self.total_cost += (input as f64 / 1_000_000.0) * in_price;
        self.total_cost += (output as f64 / 1_000_000.0) * out_price;
    }
}
