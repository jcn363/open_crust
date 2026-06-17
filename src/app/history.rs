//! Message history and session persistence management

use crate::app::App;

impl App {
    #[allow(dead_code)]
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.history_index {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_index = Some(idx);
        self.input = self.history[idx].clone();
    }

    #[allow(dead_code)]
    pub fn history_down(&mut self) {
        match self.history_index {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.history_index = None;
                self.input.clear();
            }
            Some(i) => {
                self.history_index = Some(i + 1);
                self.input = self.history[i + 1].clone();
            }
        }
    }

    fn history_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".config/opencrust/history.txt")
    }

    fn sessions_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".config/opencrust/sessions")
    }

    pub(crate) fn load_history(&mut self) {
        let path = Self::history_path();
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            self.history = content.lines().map(|l| l.to_string()).collect();
        }
    }

    pub fn save_history(&self) {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = self.history.join("\n");
        let _ = std::fs::write(path, content);
    }

    /// Auto-save the current session (messages + token budget) on exit.
    pub fn save_session(&self) {
        let dir = Self::sessions_dir();
        let _ = std::fs::create_dir_all(&dir);

        let session_file = dir.join("last_session.json");
        let session_data = serde_json::json!({
            "provider": self.config.provider.to_string(),
            "model": self.config.model,
            "token_budget": self.token_budget.as_ref().map(|b| serde_json::json!({
                "max_tokens": b.max_tokens,
                "current_tokens": b.current_tokens,
                "total_cost": b.total_cost,
            })),
        });

        if let Ok(content) = serde_json::to_string_pretty(&session_data) {
            let _ = std::fs::write(&session_file, content);
        }
    }

    /// Restore the last session's token budget if available.
    pub fn restore_session(&mut self) -> Option<String> {
        let session_file = Self::sessions_dir().join("last_session.json");
        if !session_file.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&session_file).ok()?;
        let data: serde_json::Value = serde_json::from_str(&content).ok()?;

        let provider = data.get("provider")?.as_str()?.to_string();
        let model = data.get("model")?.as_str()?.to_string();

        // Only restore if provider/model match current config
        if provider != self.config.provider.to_string() || model != self.config.model {
            return None;
        }

        // Restore token budget if present
        if let Some(budget_data) = data.get("token_budget") {
            let max_tokens = budget_data
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(1_000_000) as u32;
            let current_tokens = budget_data
                .get("current_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let total_cost = budget_data
                .get("total_cost")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let budget_key = format!("{}:{}", provider, model);
            let mut budget = crate::token_budget::TokenBudget::new(budget_key, max_tokens);
            budget.current_tokens = current_tokens;
            budget.total_cost = total_cost;
            self.token_budget = Some(budget);
        }

        Some(format!("Restored session for {}:{}", provider, model))
    }
}
