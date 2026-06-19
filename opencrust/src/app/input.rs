//! Input handling and ghost text prediction

use crate::app::App;

impl App {
    /// Check if we should trigger input prediction (300ms debounce)
    pub fn should_trigger_prediction(&self) -> bool {
        if !self.input_prediction_enabled {
            return false;
        }
        if let Some(last_time) = self.last_input_time {
            last_time.elapsed() >= std::time::Duration::from_millis(300)
        } else {
            false
        }
    }

    /// Clear ghost text and reset prediction state
    pub fn clear_ghost_text(&mut self) {
        self.ghost_text = None;
        self.last_input_time = None;
    }

    /// Mark the UI as needing a redraw
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn submit_message(&mut self) {
        if !self.input.is_empty() {
            let user_msg = self.input.clone();
            let tab_idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
            self.tabs[tab_idx]
                .messages
                .push(crate::app::Message::new(format!("You: {}", user_msg)));

            // Save to history
            if self.history.last().map(|s| s.as_str()) != Some(&user_msg) {
                self.history.push(user_msg.clone());
                self.save_history();
            }
            self.history_index = None;

            if let Some(tx) = &self.prompt_tx {
                let _ = tx.try_send(user_msg);
            }

            self.input.clear();
        }
    }

    #[cfg(test)]
    pub fn handle_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.input.pop();
    }
}
