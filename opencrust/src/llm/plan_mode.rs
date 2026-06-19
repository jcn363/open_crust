//! Plan mode functionality for LLM client

use super::LlmClient;
use super::types::PlanModeState;

impl LlmClient {
    /// Set plan mode state
    pub fn set_plan_mode(&self, mode: PlanModeState) {
        if let Ok(mut guard) = self.plan_mode.lock() {
            *guard = mode;
        }
    }

    /// Get current plan mode state
    pub fn get_plan_mode(&self) -> PlanModeState {
        self.plan_mode
            .lock()
            .map(|g| *g)
            .unwrap_or(PlanModeState::Disabled)
    }

    /// Check if a tool is blocked in plan mode
    pub(crate) fn is_tool_blocked_in_plan_mode(&self, tool_name: &str) -> bool {
        if self.get_plan_mode() != PlanModeState::Planning {
            return false;
        }
        // Block all write/modify tools in plan mode
        matches!(
            tool_name,
            "write" | "edit" | "bash" | "global_search_replace" | "create_plan"
        )
    }
}
