//! Goal management for LLM client

use super::LlmClient;
use super::types::Goal;

impl LlmClient {
    /// Set a persistent goal for autonomous execution
    pub fn set_goal(&self, description: String) {
        if let Ok(mut guard) = self.goal.lock() {
            *guard = Some(Goal {
                description,
                created_at: chrono::Utc::now(),
            });
        }
    }

    /// Clear the active goal
    pub fn clear_goal(&self) {
        if let Ok(mut guard) = self.goal.lock() {
            *guard = None;
        }
    }

    /// Get the current goal if any
    pub fn get_goal(&self) -> Option<Goal> {
        self.goal.lock().ok().and_then(|g| g.clone())
    }

    /// Get goal description for system prompt injection
    pub fn get_goal_prompt(&self) -> Option<String> {
        self.goal.lock().ok().and_then(|g| {
            g.as_ref().map(|goal| {
                format!(
                    "\n\n## Active Goal\nYou have an active goal: '{}'. Work autonomously toward completing this goal. The goal was set at {}.",
                    goal.description,
                    goal.created_at.format("%Y-%m-%d %H:%M UTC")
                )
            })
        })
    }
}