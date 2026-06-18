//! OpenCrust — Production TUI platform for AI-powered coding
//!
//! Entry point for the OpenCrust application. Parses CLI arguments via
//! clap, sets up shared subsystem managers (MCP, LSP, skills, custom tools,
//! plugins, background agents, multi-repo), dispatches to subcommand
//! handlers or launches the interactive TUI.

#![deny(warnings)]

mod acp;
mod app;
mod audit;
mod background_agents;
mod cli;
mod clipboard;
mod commands;
mod compliance;
mod config;
mod context;
mod core;
mod custom_commands;
mod custom_tools;
mod desktop;
mod event_loop;
mod events;
mod formatters;
mod git;
mod json_utils;
mod jsonrpc;
mod llm;
mod lsp;
mod manager_init;
mod markdown;
mod mcp;
mod mcp_showcase;
mod mission_control;
mod models;
mod multi_repo;
mod orchestrator;
mod permissions;
mod planner;
mod plugins;
mod pty;
mod rag;
mod rules;
mod security;
mod sessions;
mod skills;
mod startup;
mod status_bar;
mod token_budget;
mod logging;
mod tool_executor;
mod tools;
mod ui;
mod web;

use clap::Parser;
use cli::*;
use commands::dispatch;
use event_loop::run_tui;
use manager_init::{apply_cinnamon_theme, init_managers, spawn_model_refresh};
use startup::run_multi_agent;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let mut config = config::Config::load();
    apply_cinnamon_theme(&mut config);

    // Check for multi-agent mode early (before config is moved)
    if !args.agents.is_empty() {
        let prompt = match &args.multi_prompt {
            Some(p) => p.clone(),
            None => {
                eprintln!("Error: --multi-prompt is required when using --agent");
                eprintln!(
                    "Usage: opencrust --agent ollama:llama3 --agent gemini:gemini-pro --multi-prompt \"Your question\""
                );
                return Ok(());
            }
        };
        return run_multi_agent(&args.agents, &prompt, &config).await;
    }

    // Start background model list refresh if enabled
    spawn_model_refresh(&config);

    // Shared managers
    let (mcp_manager, lsp_manager, skill_manager, custom_tool_manager) =
        init_managers(&config).await;

    // Handle headless mode (--prompt) - must be before other command handling
    if let Some(ref prompt) = args.prompt {
        return run_headless(
            prompt,
            args.file.as_deref(),
            args.project.as_deref(),
            args.provider.as_deref(),
            args.model.as_deref(),
            mcp_manager,
            lsp_manager,
            skill_manager,
            custom_tool_manager,
        )
        .await;
    }

    let llm_client = llm::LlmClient::new(
        Arc::new(config.clone()),
        mcp_manager.clone(),
        lsp_manager.clone(),
        skill_manager.clone(),
        custom_tool_manager.clone(),
    )?;

    // Dispatch CLI commands
    if let Some(ref command) = args.command {
        let handled = dispatch(
            command.clone(),
            config,
            mcp_manager.clone(),
            lsp_manager.clone(),
            skill_manager.clone(),
            custom_tool_manager.clone(),
        )
        .await?;
        if handled {
            return Ok(());
        }
    }

    // ACP mode
    if matches!(args.command, Some(Commands::Acp)) {
        return acp::run_acp_loop(llm_client).await;
    }

    // Launch TUI
    let plugin_manager = llm_client.plugin_manager.clone();
    run_tui(llm_client, skill_manager, plugin_manager).await?;

    Ok(())
}

/// Run OpenCrust in headless mode (no TUI, just prompt and response)
#[expect(clippy::too_many_arguments)]
async fn run_headless(
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
