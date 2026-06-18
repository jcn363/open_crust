//! OpenCrust — Production TUI platform for AI-powered coding
//!
//! Library crate for OpenCrust. All application logic is exposed here;
//! `main.rs` is a thin CLI parser that delegates to this library.

#![deny(warnings)]

pub mod acp;
pub mod app;
pub mod audit;
pub mod background_agents;
pub mod cli;
pub mod clipboard;
pub mod commands;
pub mod compliance;
pub mod config;
pub mod context;
pub mod custom_commands;
pub mod custom_tools;
pub mod desktop;
pub mod event_loop;
pub mod events;
pub mod formatters;
pub mod git;
pub mod json_utils;
pub mod jsonrpc;
pub mod llm;
pub mod logging;
pub mod lsp;
pub mod manager_init;
pub mod markdown;
pub mod mcp;
pub mod mcp_showcase;
pub mod mission_control;
pub mod models;
pub mod multi_repo;
pub mod orchestrator;
pub mod permissions;
pub mod planner;
pub mod plugins;
pub mod providers;
pub mod pty;
pub mod rag;
pub mod rules;
pub mod security;
pub mod sessions;
pub mod skills;
pub mod startup;
pub mod status_bar;
pub mod token_budget;
pub mod tool_executor;
pub mod tools;
pub mod ui;
pub mod web;

pub use cli::*;
pub use commands::dispatch;
pub use event_loop::run_tui;
pub use manager_init::{apply_cinnamon_theme, init_managers, spawn_model_refresh};
pub use startup::run_multi_agent;

use std::sync::Arc;

/// Run OpenCrust in headless mode (no TUI, just prompt and response)
#[expect(clippy::too_many_arguments)]
pub async fn run_headless(
    prompt: &str,
    file: Option<&str>,
    project: Option<&str>,
    provider: Option<&str>,
    model: Option<&str>,
    mcp_manager: Arc<tokio::sync::Mutex<mcp::McpManager>>,
    lsp_manager: Arc<tokio::sync::Mutex<lsp::LspManager>>,
    skill_manager: Arc<tokio::sync::Mutex<skills::SkillManager>>,
    custom_tool_manager: Arc<tokio::sync::Mutex<custom_tools::CustomToolManager>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get the prompt from argument or file
    let prompt_text = if let Some(file_path) = file {
        std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read prompt file '{}': {}", file_path, e))?
    } else {
        prompt.to_string()
    };

    // Change working directory if --project is specified
    if let Some(dir) = project {
        std::env::set_current_dir(dir)
            .map_err(|e| format!("Failed to change to directory '{}': {}", dir, e))?;
        eprintln!("Working directory: {}", std::env::current_dir()?.display());
    }

    // Load and modify config
    let mut config = config::Config::load();

    // Override provider if specified
    if let Some(provider_str) = provider {
        config.provider = provider_str.parse()?;
    }

    // Override model if specified
    if let Some(model_str) = model {
        config.model = model_str.to_string();
    }

    // Create LLM client
    let llm_client = llm::LlmClient::new(
        Arc::new(config),
        mcp_manager,
        lsp_manager,
        skill_manager,
        custom_tool_manager,
    )?;

    // Send prompt and get response
    eprintln!("Sending prompt to LLM...");
    match llm_client.query_simple(&prompt_text, None).await {
        Ok(response) => {
            println!("{}", response);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
