//! Command handlers for OpenCrust CLI subcommands
//!
//! Each subcommand group has its own module with handler functions.
//! The `dispatch` function routes to the appropriate handler based on the parsed CLI.

pub mod audit;
pub mod background;
pub mod compliance;
pub mod desktop;
pub mod mcp;
pub mod plugin;
pub mod repo;
pub mod session;
pub mod skills;

use crate::cli::Commands;
use crate::config::Config;
use crate::custom_tools::CustomToolManager;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::skills::SkillManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Dispatch a CLI command to its handler.
///
/// Returns `Ok(true)` if the command was handled and the program should exit.
/// Returns `Ok(false)` if the command was not recognized (should not happen with clap).
/// Returns `Err` if the command failed.
pub async fn dispatch(
    command: Commands,
    config: Config,
    mcp_manager: Arc<Mutex<McpManager>>,
    lsp_manager: Arc<Mutex<LspManager>>,
    skill_manager: Arc<Mutex<SkillManager>>,
    custom_tool_manager: Arc<Mutex<CustomToolManager>>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match command {
        Commands::Acp => {
            // ACP is handled in main.rs before dispatch since it needs LlmClient
            Ok(false)
        }
        Commands::Run { command } => {
            crate::commands::run::handle_run(command).await?;
            Ok(true)
        }
        Commands::Mcp { cmd } => {
            mcp::handle_mcp(cmd, &config, mcp_manager).await?;
            Ok(true)
        }
        Commands::Desktop { cmd } => {
            desktop::handle_desktop(cmd).await?;
            Ok(true)
        }
        Commands::Session { cmd } => {
            session::handle_session(cmd).await?;
            Ok(true)
        }
        Commands::Audit { cmd } => {
            audit::handle_audit(cmd, &config).await?;
            Ok(true)
        }
        Commands::Compliance { cmd } => {
            compliance::handle_compliance(cmd, &config).await?;
            Ok(true)
        }
        Commands::Background { cmd } => {
            background::handle_background(
                cmd,
                &config,
                mcp_manager,
                lsp_manager,
                skill_manager,
                custom_tool_manager,
            )
            .await?;
            Ok(true)
        }
        Commands::Plugin { cmd } => {
            plugin::handle_plugin(cmd).await?;
            Ok(true)
        }
        Commands::Repo { cmd } => {
            repo::handle_repo(cmd).await?;
            Ok(true)
        }
        Commands::Skills { cmd } => {
            skills::handle_skills(cmd, skill_manager).await?;
            Ok(true)
        }
    }
}

mod run {
    use crate::security::execute_command_safely;

    pub async fn handle_run(
        command: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let output = execute_command_safely(&command)
            .map_err(|e| format!("Failed to execute command: {}", e))?;
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(output.status.code().unwrap_or(1));
        }
        Ok(())
    }
}
