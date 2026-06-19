use crate::background_agents::BackgroundAgentManager;
use crate::cli::BackgroundCommands;
use crate::config::Config;
use crate::custom_tools::CustomToolManager;
use crate::llm::LlmClient;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::skills::SkillManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_background(
    cmd: BackgroundCommands,
    _config: &Config,
    _mcp_manager: Arc<Mutex<McpManager>>,
    _lsp_manager: Arc<Mutex<LspManager>>,
    _skill_manager: Arc<Mutex<SkillManager>>,
    _custom_tool_manager: Arc<Mutex<CustomToolManager>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let agent_mgr = BackgroundAgentManager::new();
    let config = Config::load();
    match cmd {
        BackgroundCommands::List => {
            let agents = agent_mgr.list().await;
            if agents.is_empty() {
                println!("No background agents.");
            } else {
                println!(
                    "{:<5} {:<24} {:<10} {:<12} Age",
                    "ID", "Name", "Status", "Progress"
                );
                println!("{}", "-".repeat(90));
                for agent in &agents {
                    let id_short = agent.id.to_string().chars().take(8).collect::<String>();
                    let status_str = match &agent.status {
                        crate::background_agents::AgentStatus::Pending => "PENDING",
                        crate::background_agents::AgentStatus::Running => "RUNNING",
                        crate::background_agents::AgentStatus::Completed { .. } => "DONE",
                        crate::background_agents::AgentStatus::Failed { .. } => "FAILED",
                        crate::background_agents::AgentStatus::Cancelled => "CANCELLED",
                    };
                    let age = (chrono::Utc::now() - agent.created_at).num_seconds();
                    println!(
                        "{:<5} {:<24} {:<10} {:>3}%    {}s",
                        id_short, agent.name, status_str, agent.progress, age
                    );
                }
                // Also print stats
                let stats = agent_mgr.stats().await;
                println!("\n{}", stats);
            }
        }
        BackgroundCommands::Show { id } => {
            let uid = match uuid::Uuid::parse_str(&id) {
                Ok(u) => u,
                Err(_) => {
                    eprintln!("Invalid UUID: {}", id);
                    return Ok(());
                }
            };
            match agent_mgr.get(&uid).await {
                Some(agent) => {
                    println!("Agent: {}", agent.name);
                    println!("  ID:       {}", agent.id);
                    println!("  Status:   {:?}", agent.status);
                    println!("  Progress: {}%", agent.progress);
                    println!("  Created:  {}", agent.created_at);
                    if let Some(started) = agent.started_at {
                        println!("  Started:  {}", started);
                    }
                    if let Some(completed) = agent.completed_at {
                        println!("  Done:     {}", completed);
                    }
                    println!("  Logs ({}):", agent.log.len());
                    for line in agent.log.iter().rev().take(20) {
                        println!("    {}", line);
                    }
                    if let crate::background_agents::AgentStatus::Completed { output } =
                        &agent.status
                    {
                        println!("\n  Result (first 500 chars):");
                        println!("    {}", &output.chars().take(500).collect::<String>());
                    }
                    if let crate::background_agents::AgentStatus::Failed { error } = &agent.status {
                        println!("\n  Error: {}", error);
                    }
                }
                None => eprintln!("Agent '{}' not found.", id),
            }
        }
        BackgroundCommands::Start { name, prompt } => {
            // Build LlmClient for the agent
            let (mcp_mgr, lsp_mgr, skill_mgr, custom_tool_mgr) = init_managers(&config).await;
            let llm_client = match LlmClient::new(
                std::sync::Arc::new(config),
                mcp_mgr,
                lsp_mgr,
                skill_mgr,
                custom_tool_mgr,
            ) {
                Ok(c) => std::sync::Arc::new(c),
                Err(e) => {
                    eprintln!("Error creating LLM client: {}", e);
                    return Ok(());
                }
            };
            let agent_id = agent_mgr
                .spawn(name.clone(), prompt.clone(), None, None, vec![], llm_client)
                .await;
            println!("Started background agent '{}' with ID: {}", name, agent_id);
        }
        BackgroundCommands::Cancel { id } => {
            let uid = match uuid::Uuid::parse_str(&id) {
                Ok(u) => u,
                Err(_) => {
                    eprintln!("Invalid UUID: {}", id);
                    return Ok(());
                }
            };
            if agent_mgr.cancel(&uid).await {
                println!("Agent '{}' cancelled.", id);
            } else {
                eprintln!("Agent '{}' not found or already completed.", id);
            }
        }
        BackgroundCommands::Stats => {
            let stats = agent_mgr.stats().await;
            let agents = agent_mgr.list().await;
            println!("{}", stats);
            println!("\nAgents breakdown:");
            for agent in &agents {
                println!("  {}", agent);
            }
        }
        BackgroundCommands::Logs { id, lines } => {
            let uid = match uuid::Uuid::parse_str(&id) {
                Ok(u) => u,
                Err(_) => {
                    eprintln!("Invalid UUID: {}", id);
                    return Ok(());
                }
            };
            match agent_mgr.get(&uid).await {
                Some(agent) => {
                    let start = agent.log.len().saturating_sub(lines);
                    for line in agent.log.iter().skip(start) {
                        println!("{}", line);
                    }
                }
                None => eprintln!("Agent '{}' not found.", id),
            }
        }
    }
    Ok(())
}

// Helper function to initialize managers for background agents
async fn init_managers(
    config: &Config,
) -> (
    Arc<Mutex<McpManager>>,
    Arc<Mutex<LspManager>>,
    Arc<Mutex<SkillManager>>,
    Arc<Mutex<CustomToolManager>>,
) {
    let mcp_manager = Arc::new(Mutex::new(McpManager::new()));
    mcp_manager.lock().await.load_from_config(&config.mcp).await;

    let lsp_manager = Arc::new(Mutex::new(LspManager::new()));
    lsp_manager.lock().await.load_from_config(&config.lsp).await;

    let skill_manager = Arc::new(Mutex::new(SkillManager::new()));
    {
        let mut skills = skill_manager.lock().await;
        skills.discover();
    }

    let custom_tool_manager = Arc::new(Mutex::new(CustomToolManager::new()));
    {
        let mut custom = custom_tool_manager.lock().await;
        custom.discover();
    }

    (mcp_manager, lsp_manager, skill_manager, custom_tool_manager)
}
