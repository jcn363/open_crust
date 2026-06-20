//! OpenCrust — Production TUI platform for AI-powered coding
//!
//! Entry point. Thin layer over the `opencrust` library crate.

use clap::Parser;
use opencrust::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        run_multi_agent(&args.agents, &prompt, &config)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        return Ok(());
    }

    // Start background model list refresh if enabled
    spawn_model_refresh(&config);

    // Shared managers
    let (mcp_manager, lsp_manager, skill_manager, custom_tool_manager) =
        init_managers(&config).await;

    // Handle headless mode (--prompt) - must be before other command handling
    if let Some(ref prompt) = args.prompt {
        run_headless(
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
        .await?;
        return Ok(());
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
        acp::run_acp_loop(llm_client)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        return Ok(());
    }

    // Launch TUI
    let plugin_manager = llm_client.plugin_manager.clone();
    run_tui(llm_client, skill_manager, plugin_manager)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
