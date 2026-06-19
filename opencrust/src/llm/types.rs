//! Type definitions for LLM client

/// Plan mode state for read-only analysis
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PlanModeState {
    #[default]
    Disabled,
    Planning,
}

/// Persistent goal for autonomous agent execution
#[derive(Clone, Debug)]
pub struct Goal {
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub const BASE_SYSTEM_PROMPT: &str =
    "You are opencrust, a pure Rust terminal-based AI coding agent. 
You have access to tools to interact with the local filesystem and execute bash commands.
Always follow the project's rules and guidelines provided below.";
