//! Type definitions for LLM client

use crate::config::{Config, PermissionAction, ProviderType};
use crate::orchestrator::Orchestrator;
use crate::rules;
use crate::token_budget::TokenBudgetManager;
use crate::tool_executor::ToolExecutor;
use reqwest::Client;
use serde_json::{Value, json};
use std::error::Error;
use tokio::sync::mpsc;

use crate::audit::AuditLogger;
use crate::custom_tools::CustomToolManager;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::permissions::PermissionManager;
use crate::planner::Planner;
use crate::plugins::PluginManager;
use crate::rag::RagManager;
use crate::skills::SkillManager;
use crate::web::WebManager;
use async_recursion::async_recursion;
use std::sync::Arc;
use tokio::sync::Mutex;

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

pub const BASE_SYSTEM_PROMPT: &str = "You are opencrust, a pure Rust terminal-based AI coding agent. 
You have access to tools to interact with the local filesystem and execute bash commands.
Always follow the project's rules and guidelines provided below.";