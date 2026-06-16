//! Shared startup initialization for OpenCrust subsystems
//!
//! Provides helper functions for manager creation, desktop environment
//! detection, and background tasks that are used by both headless
//! and interactive modes.

use std::sync::Arc;
use tokio::sync::Mutex;

/// Parse agent spec (format: "provider:model" or just "provider")
fn parse_agent_spec(
    spec: &str,
) -> Result<(crate::config::ProviderType, String), Box<dyn std::error::Error + Send + Sync>> {
    if spec.contains(':') {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        let provider = parts[0].parse::<crate::config::ProviderType>()?;
        Ok((provider, parts[1].to_string()))
    } else {
        let provider = spec.parse::<crate::config::ProviderType>()?;
        let model = match provider {
            crate::config::ProviderType::Ollama => "deepseek-r1".to_string(),
            crate::config::ProviderType::OpenRouter => "openrouter/free".to_string(),
            crate::config::ProviderType::OpenAI => "gpt-4o-mini".to_string(),
            crate::config::ProviderType::Gemini => "gemini-2.0-flash".to_string(),
            crate::config::ProviderType::Mistral => "mistral-small".to_string(),
            crate::config::ProviderType::Anthropic => "claude-sonnet-4-20250514".to_string(),
            _ => "deepseek-r1".to_string(),
        };
        Ok((provider, model))
    }
}

/// Run multiple agents in parallel and collect responses
pub async fn run_multi_agent(
    agent_specs: &[String],
    prompt: &str,
    base_config: &crate::config::Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse agent specs
    let agents: Vec<(crate::config::ProviderType, String)> = agent_specs
        .iter()
        .map(|spec| parse_agent_spec(spec))
        .collect::<Result<Vec<_>, _>>()?;

    if agents.is_empty() {
        return Ok(());
    }

    // Create shared managers
    let mcp_manager = Arc::new(Mutex::new(crate::mcp::McpManager::new()));
    mcp_manager
        .lock()
        .await
        .load_from_config(&base_config.mcp)
        .await;

    let lsp_manager = Arc::new(Mutex::new(crate::lsp::LspManager::new()));
    lsp_manager
        .lock()
        .await
        .load_from_config(&base_config.lsp)
        .await;

    let skill_manager = Arc::new(Mutex::new(crate::skills::SkillManager::new()));
    {
        let mut skills = skill_manager.lock().await;
        skills.discover();
    }

    let custom_tool_manager = Arc::new(Mutex::new(crate::custom_tools::CustomToolManager::new()));
    {
        let mut custom = custom_tool_manager.lock().await;
        custom.discover();
    }

    // Spawn a task for each agent
    let mut handles = vec![];

    for (provider, model) in agents {
        let mut config = base_config.clone();
        config.provider = provider.clone();
        config.model = model.clone();

        let mcp_mgr = mcp_manager.clone();
        let lsp_mgr = lsp_manager.clone();
        let skill_mgr = skill_manager.clone();
        let custom_mgr = custom_tool_manager.clone();
        let prompt = prompt.to_string();

        let handle = tokio::spawn(async move {
            let provider_name = format!("{:?}", provider);
            let llm_client = match crate::llm::LlmClient::new(
                Arc::new(config),
                mcp_mgr,
                lsp_mgr,
                skill_mgr,
                custom_mgr,
            ) {
                Ok(client) => client,
                Err(e) => {
                    eprintln!(
                        "Failed to create LLM client for {} ({}): {}",
                        provider_name, model, e
                    );
                    return (provider_name, model, Err(e.to_string()));
                }
            };

            println!("Agent {} ({}) thinking...", provider_name, model);

            match llm_client.query_simple(&prompt, None).await {
                Ok(response) => (provider_name, model, Ok(response)),
                Err(e) => (provider_name, model, Err(e.to_string())),
            }
        });
        handles.push(handle);
    }

    // Wait for all agents and print results
    println!("\n=== Multi-Agent Results ===\n");

    for handle in handles {
        match handle.await {
            Ok((provider, model, result)) => {
                println!("## {} ({})", provider, model);
                match result {
                    Ok(response) => println!("{}\n", response),
                    Err(e) => eprintln!("Error: {}\n", e),
                }
                println!("{}", "-".repeat(50));
            }
            Err(e) => eprintln!("Task error: {}", e),
        }
    }

    Ok(())
}
