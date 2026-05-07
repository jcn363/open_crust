#![deny(warnings)]
mod acp;
mod app;
mod audit;
mod clipboard;
mod config;
mod context;
mod custom_tools;
mod desktop;
mod events;
mod formatters;
mod git;
mod json_utils;
mod jsonrpc;
mod llm;
mod lsp;
mod markdown;
mod mcp;
mod permissions;
mod planner;
mod rag;
mod rules;
mod security;
mod sessions;
mod skills;
mod stats;
mod telemetry;
mod tool_executor;
mod tools;
mod ui;
mod web;

use desktop::detection::get_cinnamon_info;

use app::{App, Mode};
use clap::{Parser, Subcommand};
use clipboard::ClipboardManager;
use crossterm::{
    ExecutableCommand,
    event::{Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde_json::Value;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run with multiple agents in parallel (format: provider:model, e.g., ollama:llama3)
    #[arg(long, value_name = "AGENT", num_args = 0..)]
    agents: Vec<String>,

    /// Prompt to send to multiple agents (use with --agent)
    #[arg(long, value_name = "PROMPT")]
    multi_prompt: Option<String>,

    /// Run in headless mode with a single prompt (no TUI)
    #[arg(short = 'p', long, value_name = "PROMPT")]
    prompt: Option<String>,

    /// Read prompt from file (use with --prompt)
    #[arg(short = 'f', long, value_name = "FILE")]
    file: Option<String>,

    /// Set working directory for headless mode
    #[arg(long, value_name = "DIR")]
    project: Option<String>,

    /// Override provider for this invocation
    #[arg(long, value_name = "PROVIDER")]
    provider: Option<String>,

    /// Override model for this invocation
    #[arg(long, value_name = "MODEL")]
    model: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start in ACP mode (JSON-RPC over stdio)
    Acp,
    /// Run a single command and exit
    Run { command: String },
    /// MCP server management
    Mcp {
        #[command(subcommand)]
        cmd: McpCommands,
    },
    /// Desktop integration features (file picker, notifications)
    Desktop {
        #[command(subcommand)]
        cmd: DesktopCommands,
    },
    /// Session management
    Session {
        #[command(subcommand)]
        cmd: SessionCommands,
    },
    /// Skill management
    Skills {
        #[command(subcommand)]
        cmd: SkillsCommands,
    },
    /// Run as an MCP server (expose OpenCrust tools via MCP protocol)
    Serve {
        /// Port to listen on (default: 8765)
        #[arg(short, long, default_value = "8765")]
        port: u16,
        /// Use stdio transport instead of TCP
        #[arg(long)]
        stdio: bool,
    },
}

#[derive(Subcommand, Debug)]
enum McpCommands {
    /// List available MCP servers
    List,
    /// Install an MCP server by name
    Install { server: String },
}

#[derive(Subcommand, Debug)]
enum DesktopCommands {
    /// Open file picker dialog
    FilePicker {
        /// Mode: open, open-multiple, save, directory
        #[arg(short, long, default_value = "open")]
        mode: String,
        /// Initial directory
        #[arg(short, long)]
        dir: Option<String>,
        /// Window title
        #[arg(short, long)]
        title: Option<String>,
    },
    /// Send a desktop notification
    Notify {
        /// Notification title
        #[arg(short, long)]
        title: String,
        /// Notification body
        #[arg(short, long)]
        body: String,
        /// Urgency: low, normal, critical
        #[arg(short, long, default_value = "normal")]
        urgency: String,
    },
    /// Detect desktop environment
    Detect,
}

#[derive(Subcommand, Debug)]
enum SessionCommands {
    /// List all sessions
    List,
    /// Show a specific session
    Show { id: String },
    /// Delete a session
    Delete { id: String },
    /// Save current session (requires messages JSON)
    Save {
        id: String,
        #[arg(short, long)]
        messages: String,
    },
    /// Fork a session to experiment with different approaches
    Fork {
        id: String,
        #[arg(short, long)]
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum SkillsCommands {
    /// List all skills with their status
    List,
    /// Activate a skill
    Activate { name: String },
    /// Deactivate a skill
    Deactivate { name: String },
    /// Show skill statistics
    Stats { name: Option<String> },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let mut config = config::Config::load();

    // Detect Cinnamon desktop environment and apply theming if available
    let cinnamon_info = get_cinnamon_info();
    if cinnamon_info.desktop.is_cinnamon() {
        // Log desktop detection (silent, for debugging)
        eprintln!(
            "[Desktop] Detected: {} {}",
            cinnamon_info.desktop,
            cinnamon_info.version.as_deref().unwrap_or("")
        );
    }

    // If no custom theme is configured, apply Cinnamon theme
    if config.theme.is_none() && cinnamon_info.desktop.is_cinnamon() {
        config.theme = Some(config::ThemeConfig {
            background: cinnamon_info.theme.background.clone(),
            foreground: cinnamon_info.theme.foreground.clone(),
            accent: cinnamon_info.theme.accent.clone(),
            border: cinnamon_info.theme.border.clone(),
        });
    }

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

    // Shared managers
    let mcp_manager = Arc::new(Mutex::new(mcp::McpManager::new()));
    mcp_manager.lock().await.load_from_config(&config.mcp).await;

    let lsp_manager = Arc::new(Mutex::new(lsp::LspManager::new()));
    lsp_manager.lock().await.load_from_config(&config.lsp).await;

    let skill_manager = Arc::new(Mutex::new(skills::SkillManager::new()));
    {
        let mut skills = skill_manager.lock().await;
        skills.discover();
    }

    let custom_tool_manager = Arc::new(Mutex::new(custom_tools::CustomToolManager::new()));
    {
        let mut custom = custom_tool_manager.lock().await;
        custom.discover();
    }

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
        config,
        mcp_manager.clone(),
        lsp_manager.clone(),
        skill_manager.clone(),
        custom_tool_manager.clone(),
    );

    if let Some(Commands::Acp) = args.command {
        return acp::run_acp_loop(llm_client).await;
    }

    if let Some(Commands::Run { command }) = args.command {
        println!("Running command: {}", command);
        return Ok(());
    }

    if let Some(Commands::Mcp { cmd }) = &args.command {
        match cmd {
            McpCommands::List => {
                println!("Available MCP servers (curated list):");
                println!("\n=== Tier 1: Essential ===");
                println!(
                    "  context7         - Version-accurate library docs (eliminates API hallucinations)"
                );
                println!("  github          - GitHub integration (repos, issues, PRs, CI/CD)");
                println!("  postgres        - PostgreSQL database queries");
                println!("  brave-search    - Web search (privacy-focused)");
                println!("  filesystem      - Enhanced file system access");
                println!("  sequentialthinking - Structured thinking and reasoning");
                println!("\n=== Tier 2: High Value ===");
                println!("  playwright      - Browser automation & E2E testing");
                println!("  supabase        - RLS-aware database access");
                println!("  sentry          - Error monitoring integration");
                println!("  linear          - Issue tracking");
                println!("  e2b             - Secure cloud sandbox for code execution");
                println!("  octocode        - Code analysis and refactoring");
                println!("\n=== Tier 3: Production ===");
                println!("  slack           - Slack messaging");
                println!("  google-drive    - Google Drive file access");
                println!("  stripe          - Payment integration (requires OAuth)");
                println!("\nUse `opencrust mcp install <name>` to add a server.");
                println!(
                    "For more servers, visit: https://github.com/modelcontextprotocol/servers"
                );
                println!("Or browse: https://mcpdirectory.app/ (2,500+ servers)");
            }
            McpCommands::Install { server } => {
                let config = config::Config::load();
                let mut new_config = config.clone();
                let (command, description, env_help) = match server.as_str() {
                    // Tier 1: Essential
                    "context7" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@context7/mcp-server".to_string(),
                        ],
                        "Version-accurate library docs".to_string(),
                        "No API key required",
                    ),
                    "github" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-github".to_string(),
                        ],
                        "GitHub integration (repos, issues, PRs)".to_string(),
                        "Set GITHUB_TOKEN env var",
                    ),
                    "postgres" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-postgres".to_string(),
                        ],
                        "PostgreSQL database queries".to_string(),
                        "Set DATABASE_URL env var",
                    ),
                    "brave-search" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-brave-search".to_string(),
                        ],
                        "Web search (privacy-focused)".to_string(),
                        "Set BRAVE_API_KEY env var",
                    ),
                    "filesystem" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-filesystem".to_string(),
                        ],
                        "Enhanced file system access".to_string(),
                        "Set ALLOWED_DIRS env var",
                    ),
                    "sequentialthinking" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-sequential-thinking".to_string(),
                        ],
                        "Structured thinking and reasoning".to_string(),
                        "No API key required",
                    ),
                    // Tier 2: High Value
                    "playwright" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-playwright".to_string(),
                        ],
                        "Browser automation & E2E testing".to_string(),
                        "Run: npx playwright install",
                    ),
                    "supabase" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@supabase/mcp-server-supabase".to_string(),
                        ],
                        "RLS-aware database access".to_string(),
                        "Set SUPABASE_ACCESS_TOKEN env var",
                    ),
                    "sentry" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-sentry".to_string(),
                        ],
                        "Error monitoring integration".to_string(),
                        "Set SENTRY_AUTH_TOKEN env var",
                    ),
                    "linear" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-linear".to_string(),
                        ],
                        "Issue tracking".to_string(),
                        "Set LINEAR_API_KEY env var",
                    ),
                    "e2b" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@e2b/mcp-server".to_string(),
                        ],
                        "Secure cloud sandbox for code execution".to_string(),
                        "Set E2B_API_KEY env var",
                    ),
                    "octocode" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@octocode/mcp-server".to_string(),
                        ],
                        "Code analysis and refactoring".to_string(),
                        "No API key required",
                    ),
                    // Tier 3: Production
                    "slack" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-slack".to_string(),
                        ],
                        "Slack messaging".to_string(),
                        "Set SLACK_TOKEN env var",
                    ),
                    "google-drive" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-google-drive".to_string(),
                        ],
                        "Google Drive file access".to_string(),
                        "OAuth required",
                    ),
                    "stripe" => (
                        vec![
                            "npx".to_string(),
                            "-y".to_string(),
                            "@modelcontextprotocol/server-stripe".to_string(),
                        ],
                        "Payment integration".to_string(),
                        "Set STRIPE_API_KEY env var",
                    ),
                    _ => {
                        eprintln!(
                            "Unknown MCP server: {}. Use `opencrust mcp list` to see available servers.",
                            server
                        );
                        return Ok(());
                    }
                };
                let mcp_config = config::McpConfig {
                    command,
                    environment: None,
                    enabled: true,
                };
                new_config.mcp.insert(server.clone(), mcp_config);
                new_config.save();
                println!("Installed MCP server '{}'.", server);
                println!("  Description: {}", description);
                println!("  Setup: {}", env_help);
                println!("\nRestart open_crust to use the server.");
            }
        }
        return Ok(());
    }

    // Desktop integration commands
    if let Some(Commands::Desktop { cmd }) = &args.command {
        match cmd {
            DesktopCommands::FilePicker { mode, dir, title } => {
                use desktop::file_picker::{
                    FilePickerBackend, FilePickerMode, FilePickerOptions,
                    detect_file_picker_backend, file_picker,
                };

                let backend = detect_file_picker_backend();
                if backend == FilePickerBackend::None {
                    eprintln!(
                        "Error: No file picker backend available (need nemo, zenity, or kdialog)"
                    );
                    return Ok(());
                }

                let mode = match mode.as_str() {
                    "open" => FilePickerMode::OpenFile,
                    "open-multiple" => FilePickerMode::OpenMultiple,
                    "save" => FilePickerMode::Save,
                    "directory" => FilePickerMode::Directory,
                    _ => {
                        eprintln!(
                            "Invalid mode: {}. Use: open, open-multiple, save, directory",
                            mode
                        );
                        return Ok(());
                    }
                };

                let options = FilePickerOptions {
                    initial_dir: dir.as_ref().map(std::path::PathBuf::from),
                    title: title.clone(),
                    ..Default::default()
                };

                let result = file_picker(mode, &options);
                if result.cancelled {
                    println!("Cancelled");
                } else {
                    for path in result.paths {
                        println!("{}", path.display());
                    }
                }
            }
            DesktopCommands::Notify {
                title,
                body,
                urgency,
            } => {
                use desktop::notifications::{
                    Notification, NotificationUrgency, send_notification_smart,
                };

                let urgency = NotificationUrgency::from_str(urgency);
                let notification = Notification::new(title, body).with_urgency(urgency);

                match send_notification_smart(&notification) {
                    Ok(_) => println!("Notification sent: {} - {}", title, body),
                    Err(e) => eprintln!("Failed to send notification: {}", e),
                }
            }
            DesktopCommands::Detect => {
                use desktop::detection::{
                    detect_desktop, detect_display_server, get_cinnamon_info, is_supported_desktop,
                };

                let desktop = detect_desktop();
                let display_server = detect_display_server();
                println!("Desktop environment: {}", desktop);
                println!("Display server: {}", display_server.name());
                println!("Supported: {}", is_supported_desktop());

                if desktop.is_cinnamon() {
                    let info = get_cinnamon_info();
                    println!("\nCinnamon Info:");
                    println!(
                        "  Version: {}",
                        info.version.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "  Theme: background={}, foreground={}",
                        info.theme.background, info.theme.foreground
                    );
                    println!("  Accent: {}", info.theme.accent);
                    println!("  Icon theme: {}", info.icon_theme);
                    println!("  Cursor theme: {}", info.cursor_theme);
                    println!("  Display server: {}", info.display_server.name());
                }
            }
        }
        return Ok(());
    }

    // Session management commands
    if let Some(Commands::Session { cmd }) = &args.command {
        let session_manager = sessions::SessionManager::new();

        match cmd {
            SessionCommands::List => {
                let sessions = session_manager.list_sessions();
                if sessions.is_empty() {
                    println!("No sessions found.");
                } else {
                    println!("Sessions:");
                    for session in sessions {
                        println!(
                            "  {} - {} messages (created: {})",
                            session.id,
                            session.messages.len(),
                            session.timestamp.format("%Y-%m-%d %H:%M:%S")
                        );
                    }
                }
            }
            SessionCommands::Show { id } => match session_manager.load_session(id) {
                Ok(session) => {
                    println!("Session: {}", session.id);
                    println!("Created: {}", session.timestamp);
                    println!("Messages: {}", session.messages.len());
                    println!("\n--- Messages ---");
                    for (i, msg) in session.messages.iter().enumerate() {
                        let role = msg
                            .get("role")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let content = msg
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .chars()
                            .take(80)
                            .collect::<String>();
                        println!("{}. [{}] {}", i + 1, role, content);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            },
            SessionCommands::Delete { id } => match session_manager.delete_session(id) {
                Ok(_) => println!("Session deleted."),
                Err(e) => eprintln!("Error: {}", e),
            },
            SessionCommands::Save { id, messages } => {
                let msgs: Vec<Value> = serde_json::from_str(messages)
                    .map_err(|e| format!("Invalid JSON: {}", e))
                    .unwrap_or_default();

                match session_manager.save_session(id, &msgs) {
                    Ok(_) => println!("Session '{}' saved ({} messages).", id, msgs.len()),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            SessionCommands::Fork { id, name } => {
                match session_manager.fork_session(id, name.as_deref()) {
                    Ok(new_session) => {
                        println!("Forked session '{}' → '{}'", id, new_session.id);
                        println!("Timestamp: {}", new_session.timestamp);
                        println!("Messages copied: {}", new_session.messages.len());
                    }
                    Err(e) => {
                        eprintln!("Error forking session: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        return Ok(());
    }
    // Skill management commands
    if let Some(Commands::Skills { cmd }) = &args.command {
        let mut skills = skill_manager.lock().await;
        match cmd {
            SkillsCommands::List => {
                println!("Skills (use 'opencrust skills activate/deactivate <name>' to toggle):");
                println!();
                for (name, description, active, usage_count, avg_latency) in
                    skills.list_skills_with_stats()
                {
                    let status = if active { "ACTIVE" } else { "inactive" };
                    println!(
                        "[{}] {} - {} (usage: {}, avg latency: {}ms)",
                        status, name, description, usage_count, avg_latency
                    );
                }
            }
            SkillsCommands::Activate { name } => {
                if skills.activate_skill(name) {
                    println!("Skill '{}' activated.", name);
                } else {
                    eprintln!("Skill '{}' not found.", name);
                }
            }
            SkillsCommands::Deactivate { name } => {
                if skills.deactivate_skill(name) {
                    println!("Skill '{}' deactivated.", name);
                } else {
                    eprintln!("Skill '{}' not found.", name);
                }
            }
            SkillsCommands::Stats { name } => {
                if let Some(skill_name) = name {
                    match skills.get_skill(skill_name) {
                        Some(skill) => {
                            println!("Skill: {}", skill.metadata.name);
                            println!("Description: {}", skill.metadata.description);
                            println!(
                                "Status: {}",
                                if skill.active { "ACTIVE" } else { "inactive" }
                            );
                            println!("Usage count: {}", skill.usage_count);
                            println!("Total latency: {}ms", skill.total_latency_ms);
                            println!("Average latency: {}ms", skill.avg_latency_ms());
                        }
                        None => eprintln!("Skill '{}' not found.", skill_name),
                    }
                } else {
                    // Show stats for all skills
                    println!("Skill Statistics:");
                    println!();
                    for (name, _, active, usage_count, avg_latency) in
                        skills.list_skills_with_stats()
                    {
                        println!(
                            "{}: usage={}, avg_latency={}ms, status={}",
                            name,
                            usage_count,
                            avg_latency,
                            if active { "active" } else { "inactive" }
                        );
                    }
                }
            }
        }
        return Ok(());
    }

    // Handle Serve command (MCP server mode)
    if let Some(Commands::Serve { port, stdio }) = &args.command {
        return mcp::run_mcp_server(*port, *stdio).await;
    }

    let (prompt_tx, mut prompt_rx) = mpsc::channel::<String>(32);
    let (response_tx, mut response_rx) = mpsc::channel::<String>(32);
    let (approval_tx, mut approval_rx) = mpsc::channel::<bool>(1);
    let (background_task_tx, mut background_task_rx) = mpsc::channel::<String>(32);
    let (prediction_tx, mut prediction_rx) = mpsc::channel::<(String, String)>(32); // (input_text, prediction)

    let mut client_clone = llm_client.clone();
    tokio::spawn(async move {
        let mut messages_history: Vec<Value> = Vec::new();
        while let Some(prompt) = prompt_rx.recv().await {
            let prompt_str = prompt.trim();

            if prompt_str == "/init" {
                match rules::init_project_rules() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("open_crust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/provider ") {
                let new_provider = prompt_str.trim_start_matches("/provider ").trim();
                match new_provider.to_lowercase().as_str() {
                    "ollama" => {
                        client_clone.config.provider = config::ProviderType::Ollama;
                        client_clone.config.save();
                        let _ = response_tx
                            .send("open_crust: Provider switched to Ollama".to_string())
                            .await;
                    }
                    "openrouter" => {
                        client_clone.config.provider = config::ProviderType::OpenRouter;
                        client_clone.config.save();
                        let _ = response_tx
                            .send("open_crust: Provider switched to OpenRouter".to_string())
                            .await;
                    }
                    _ => {
                        let _ = response_tx
                            .send(format!("open_crust: Unknown provider '{}'", new_provider))
                            .await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/model ") {
                let new_model = prompt_str.trim_start_matches("/model ").trim();
                client_clone.config.model = new_model.to_string();
                client_clone.config.save();
                let _ = response_tx
                    .send(format!("open_crust: Model switched to '{}'", new_model))
                    .await;
                continue;
            } else if prompt_str == "/undo" {
                match git::undo() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("open_crust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            } else if prompt_str == "/redo" {
                match git::redo() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("open_crust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            }

            let _ = git::checkpoint();
            let enriched_prompt = context::inject_file_context(&prompt);
            let _ = response_tx
                .send(String::from("open_crust: Thinking..."))
                .await;

            let res = client_clone
                .send_message(
                    &mut messages_history,
                    &enriched_prompt,
                    response_tx.clone(),
                    &mut approval_rx,
                )
                .await;
            match res {
                Ok(reply) => {
                    let _ = response_tx.send(format!("open_crust: {}", reply)).await;
                }
                Err(e) => {
                    let _ = response_tx.send(format!("Error: {}", e)).await;
                }
            }
        }
    });

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new(
        llm_client.config.clone(),
        prompt_tx,
        approval_tx,
        background_task_tx,
        llm_client.clone(),
    );
    app.refresh_sidebar();

    // Populate skill browser items from skill manager
    {
        let skills = skill_manager.lock().await;
        let skill_list = skills.list_skills_with_stats();
        for (name, description, active, usage_count, avg_latency) in skill_list {
            app.skill_browser_items
                .push((name, description, active, usage_count, avg_latency));
        }
    }

    // Initialize clipboard manager
    let mut clipboard = ClipboardManager::new();

    // Get copy/paste keybinds from config
    let keybinds = app.config.tui.as_ref().map(|t| &t.keybinds);
    let copy_key = keybinds
        .map(|k| k.copy.clone())
        .unwrap_or_else(|| "ctrl+c".to_string());
    let paste_key = keybinds
        .map(|k| k.paste.clone())
        .unwrap_or_else(|| "ctrl+v".to_string());
    let exit_keys = keybinds
        .map(|k| k.app_exit.clone())
        .unwrap_or_else(|| "ctrl+q".to_string());
    let submit_keys = keybinds
        .map(|k| k.input_submit.clone())
        .unwrap_or_else(|| "return".to_string());

    loop {
        while let Ok(response) = response_rx.try_recv() {
            if response.contains("[APPROVAL_REQUIRED]") {
                app.waiting_for_approval = true;
            } else if response.starts_with("[DIFF_REQUIRED]") {
                let parts: Vec<&str> = response
                    .strip_prefix("[DIFF_REQUIRED]")
                    .unwrap()
                    .splitn(3, '|')
                    .collect();
                if parts.len() == 3 {
                    app.proposed_changes.push(crate::app::ProposedChange {
                        path: parts[0].to_string(),
                        original: parts[1].to_string(),
                        proposed: parts[2].to_string(),
                        status: crate::app::ChangeStatus::Pending,
                    });
                    app.mode = crate::app::Mode::Review;
                }
            }

            let tab = &mut app.tabs[0]; // always push to Chat tab
            if let Some(last) = tab.messages.last()
                && (last == "open_crust: Thinking..."
                    || last.starts_with("open_crust: Executing tool"))
            {
                tab.messages.pop();
            }
            tab.messages.push(response);
        }

        // Handle background task notifications
        while let Ok(notification) = background_task_rx.try_recv() {
            if notification.starts_with("[TASK_COMPLETE]") {
                let parts: Vec<&str> = notification
                    .strip_prefix("[TASK_COMPLETE]")
                    .unwrap()
                    .splitn(2, "::")
                    .collect();
                if parts.len() == 2 {
                    let task_id = parts[0].to_string();
                    let result = parts[1].to_string();
                    // Update task status
                    if let Some(task) = app.background_tasks.iter_mut().find(|t| t.id == task_id) {
                        task.status = crate::app::TaskStatus::Completed;
                        task.result = Some(result.clone());
                    }
                    // Add to tasks tab
                    let tasks_tab = &mut app.tabs[1];
                    tasks_tab
                        .messages
                        .push(format!("Task {} completed: {}", task_id, result));
                }
            } else if notification.starts_with("[TASK_FAILED]") {
                let parts: Vec<&str> = notification
                    .strip_prefix("[TASK_FAILED]")
                    .unwrap()
                    .splitn(2, "::")
                    .collect();
                if parts.len() == 2 {
                    let task_id = parts[0].to_string();
                    let error = parts[1].to_string();
                    // Update task status
                    if let Some(task) = app.background_tasks.iter_mut().find(|t| t.id == task_id) {
                        task.status = crate::app::TaskStatus::Failed;
                        task.result = Some(error.clone());
                    }
                    // Add to tasks tab
                    let tasks_tab = &mut app.tabs[1];
                    tasks_tab
                        .messages
                        .push(format!("Task {} failed: {}", task_id, error));
                }
            }
        }

        terminal.draw(|f| ui::draw(f, &app))?;

        if let Some(Event::Key(key)) = events::next_event().await? {
            // Check for Copy (Ctrl+C) - copy current input to clipboard
            if check_key_match(&key, &copy_key) {
                if !app.input.is_empty() && clipboard.copy(&app.input) {
                    app.tabs[0]
                        .messages
                        .push(String::from("Copied to clipboard"));
                }
                continue;
            }

            // Check for Paste (Ctrl+V) - paste from clipboard to input
            if check_key_match(&key, &paste_key) {
                if let Some(text) = clipboard.paste() {
                    app.input.push_str(&text);
                    // Update last input time for prediction
                    app.last_input_time = Some(std::time::Instant::now());
                }
                continue;
            }

            if app.waiting_for_approval {
                match key.code {
                    KeyCode::Char('y' | 'Y') => {
                        app.waiting_for_approval = false;
                        if let Some(tx) = &app.approval_tx {
                            let _ = tx.try_send(true);
                        }
                        app.tabs[0].messages.push(String::from("You: y (Approved)"));
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.waiting_for_approval = false;
                        if let Some(tx) = &app.approval_tx {
                            let _ = tx.try_send(false);
                        }
                        app.tabs[0].messages.push(String::from("You: n (Denied)"));
                    }
                    _ => {}
                }
            } else {
                // Check for exit keys
                if check_key_match(&key, &exit_keys) {
                    app.should_quit = true;
                }

                // Global keys
                if key.modifiers == crossterm::event::KeyModifiers::CONTROL
                    && key.code == KeyCode::Char('b')
                {
                    app.show_sidebar = !app.show_sidebar;
                    continue;
                }

                match app.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('i') => {
                            app.enter_insert_mode();
                        }
                        KeyCode::Char('s') => {
                            app.mode = Mode::Servers;
                        }
                        KeyCode::Char('p')
                            if key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                        {
                            app.plan_mode = crate::app::PlanMode::Planning;
                            app.tabs[0]
                                .messages
                                .push(String::from("Entering plan mode..."));
                        }
                        KeyCode::Char('k')
                            if key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                        {
                            app.mode = Mode::CommandPalette;
                        }
                        KeyCode::Char('K')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL)
                                && key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::SHIFT) =>
                        {
                            app.mode = Mode::SkillBrowser;
                        }
                        KeyCode::Char('t')
                            if key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                        {
                            // Spawn background task with current input
                            if !app.input.is_empty() {
                                let prompt = app.input.clone();
                                app.spawn_background_task(prompt);
                                app.input.clear();
                                app.tabs[0]
                                    .messages
                                    .push(String::from("Spawning background task..."));
                            } else {
                                app.tabs[0]
                                    .messages
                                    .push(String::from("No input to spawn as background task"));
                            }
                        }
                        KeyCode::Tab => {
                            app.active_tab = (app.active_tab + 1) % app.tabs.len();
                        }
                        _ => {}
                    },
                    Mode::Insert => {
                        if check_key_match(&key, &submit_keys) {
                            app.submit_message();
                        } else {
                            match key.code {
                                KeyCode::Tab => {
                                    // Accept ghost text prediction
                                    if let Some(ghost) = app.ghost_text.take() {
                                        app.input.push_str(&ghost);
                                        // Update last_input_time to prevent immediate re-prediction
                                        app.last_input_time = Some(std::time::Instant::now());
                                    }
                                }
                                KeyCode::Esc => {
                                    // Dismiss ghost text if present, otherwise exit Insert mode
                                    if app.ghost_text.is_some() {
                                        app.clear_ghost_text();
                                    } else {
                                        app.enter_normal_mode();
                                    }
                                }
                                KeyCode::Backspace => {
                                    app.handle_backspace();
                                }
                                KeyCode::Char(c) => {
                                    app.handle_char(c);
                                    // Update last input time for prediction
                                    app.last_input_time = Some(std::time::Instant::now());
                                }
                                KeyCode::Up => {
                                    app.history_up();
                                }
                                KeyCode::Down => {
                                    app.history_down();
                                }
                                _ => {}
                            }
                        }
                    }
                    Mode::Review => match key.code {
                        // Navigation
                        KeyCode::Up if app.plan_review_index > 0 => {
                            app.plan_review_index -= 1;
                        }
                        KeyCode::Down if app.plan_review_index + 1 < app.proposed_changes.len() => {
                            app.plan_review_index += 1;
                        }
                        // Approve current file
                        KeyCode::Char('a') => {
                            if let Some(change) =
                                app.proposed_changes.get_mut(app.plan_review_index)
                            {
                                change.status = crate::app::ChangeStatus::Approved;
                            }
                        }
                        // Deny current file
                        KeyCode::Char('d') => {
                            if let Some(change) =
                                app.proposed_changes.get_mut(app.plan_review_index)
                            {
                                change.status = crate::app::ChangeStatus::Denied;
                            }
                        }
                        // Approve all files (Shift+A)
                        KeyCode::Char('A')
                            if key.modifiers == crossterm::event::KeyModifiers::SHIFT =>
                        {
                            for change in &mut app.proposed_changes {
                                change.status = crate::app::ChangeStatus::Approved;
                            }
                        }
                        // Execute approved changes
                        KeyCode::Enter => {
                            // Execute approved changes
                            let approved: Vec<_> = app
                                .proposed_changes
                                .iter()
                                .filter(|c| c.status == crate::app::ChangeStatus::Approved)
                                .cloned()
                                .collect();

                            for change in &approved {
                                // Write approved changes to files
                                if let Err(e) = std::fs::write(&change.path, &change.proposed) {
                                    app.tabs[0]
                                        .messages
                                        .push(format!("Error writing {}: {}", change.path, e));
                                } else {
                                    app.tabs[0]
                                        .messages
                                        .push(format!("Applied: {}", change.path));
                                }
                            }

                            // Clear reviewed changes
                            app.proposed_changes.clear();
                            app.plan_review_index = 0;
                            app.mode = Mode::Normal;
                            app.tabs[0]
                                .messages
                                .push(format!("Executed {} approved changes", approved.len()));
                        }
                        // Cancel (Esc)
                        KeyCode::Esc => {
                            app.proposed_changes.clear();
                            app.plan_review_index = 0;
                            app.mode = Mode::Normal;
                            app.tabs[0].messages.push(String::from("Plan cancelled"));
                        }
                        _ => {}
                    },
                    Mode::Servers => match key.code {
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Up => {
                            if app.mcp_browser_selected > 0 {
                                app.mcp_browser_selected -= 1;
                            }
                            // Adjust scroll if needed
                            if app.mcp_browser_selected < app.mcp_browser_scroll {
                                app.mcp_browser_scroll = app.mcp_browser_selected;
                            }
                        }
                        KeyCode::Down => {
                            if app.mcp_browser_selected < app.mcp_browser_items.len() - 1 {
                                app.mcp_browser_selected += 1;
                            }
                            // Adjust scroll if needed (assuming 20 visible items)
                            if app.mcp_browser_selected >= app.mcp_browser_scroll + 20 {
                                app.mcp_browser_scroll = app.mcp_browser_selected - 19;
                            }
                        }
                        KeyCode::Enter => {
                            // Install the selected server
                            if let Some((name, _, cmd)) =
                                app.mcp_browser_items.get(app.mcp_browser_selected)
                                && !app.config.mcp.contains_key(name)
                            {
                                let mcp_config = config::McpConfig {
                                    command: cmd.clone(),
                                    environment: None,
                                    enabled: true,
                                };
                                app.config.mcp.insert(name.clone(), mcp_config);
                                app.config.save();
                                app.tabs[0].messages.push(format!("System: Installed MCP server '{}'. Restart open_crust to use it.", name));
                            }
                        }
                        KeyCode::Char(c) => {
                            app.mcp_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.mcp_input.pop();
                        }
                        _ => {}
                    },
                    Mode::SkillBrowser => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Up => {
                            if app.skill_browser_selected > 0 {
                                app.skill_browser_selected -= 1;
                            }
                            // Adjust scroll if needed
                            if app.skill_browser_selected < app.skill_browser_scroll {
                                app.skill_browser_scroll = app.skill_browser_selected;
                            }
                        }
                        KeyCode::Down => {
                            if app.skill_browser_selected < app.skill_browser_items.len() - 1 {
                                app.skill_browser_selected += 1;
                            }
                            // Adjust scroll if needed (assuming 20 visible items)
                            if app.skill_browser_selected >= app.skill_browser_scroll + 20 {
                                app.skill_browser_scroll = app.skill_browser_selected - 19;
                            }
                        }
                        KeyCode::Enter => {
                            // Toggle skill active state
                            if let Some((name, _, active, _, _)) =
                                app.skill_browser_items.get_mut(app.skill_browser_selected)
                            {
                                let new_active = !*active;
                                *active = new_active;

                                // Update skill_manager (clone before moving into async block)
                                let skill_name = name.clone();
                                let sm = skill_manager.clone();
                                tokio::spawn(async move {
                                    let mut skills = sm.lock().await;
                                    if new_active {
                                        let _ = skills.activate_skill(skill_name.as_str());
                                    } else {
                                        let _ = skills.deactivate_skill(skill_name.as_str());
                                    }
                                });

                                let status = if new_active {
                                    "activated"
                                } else {
                                    "deactivated"
                                };
                                app.tabs[0]
                                    .messages
                                    .push(format!("System: Skill '{}' {}", name, status));
                            }
                        }
                        _ => {}
                    },
                    Mode::CommandPalette => match key.code {
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Up if app.command_palette_selected > 0 => {
                            app.command_palette_selected -= 1;
                        }
                        KeyCode::Down if app.command_palette_selected < 4 => {
                            // 5 items total (0-4)
                            app.command_palette_selected += 1;
                        }
                        KeyCode::Enter => {
                            // Handle command palette selection
                            match app.command_palette_selected {
                                0 => {
                                    // Switch Provider
                                    // Cycle through providers
                                    let providers = [
                                        config::ProviderType::Ollama,
                                        config::ProviderType::OpenRouter,
                                        config::ProviderType::OpenAI,
                                        config::ProviderType::Gemini,
                                        config::ProviderType::Mistral,
                                        config::ProviderType::Anthropic,
                                    ];
                                    let current_idx = providers
                                        .iter()
                                        .position(|p| p == &app.config.provider)
                                        .unwrap_or(0);
                                    let next_idx = (current_idx + 1) % providers.len();
                                    app.config.provider = providers[next_idx].clone();
                                    app.config.save();
                                    app.tabs[0].messages.push(format!(
                                        "Provider switched to {:?}",
                                        app.config.provider
                                    ));
                                    app.mode = Mode::Normal;
                                }
                                1 => {
                                    // Switch Model
                                    // For simplicity, just show a message - in a real implementation this would open a model selector
                                    app.tabs[0].messages.push(format!("Model switching not fully implemented yet. Current model: {}", app.config.model));
                                    app.mode = Mode::Normal;
                                }
                                2 => {
                                    // Show Stats
                                    // Show usage stats
                                    let stats = app.llm_client.usage_stats.blocking_lock();
                                    app.tabs[0].messages.push(format!(
                                        "Tokens: {} in, {} out | Cost: ${:.4}",
                                        stats.input_tokens, stats.output_tokens, stats.total_cost
                                    ));
                                    app.mode = Mode::Normal;
                                }
                                3 => {
                                    // Clear Context
                                    app.tabs[0].messages.clear();
                                    app.tabs[0].messages.push(String::from("Welcome to open_crust. Press 'i' to enter insert mode, 's' for servers, 'q' to quit."));
                                    app.history.clear();
                                    app.save_history();
                                    app.tabs[0].messages.push("Context cleared.".to_string());
                                    app.mode = Mode::Normal;
                                }
                                4 => {
                                    // MCP Browser
                                    app.mode = Mode::Servers;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                }
            }
        }

        // Check for prediction results
        if let Ok((input_text, prediction)) = prediction_rx.try_recv()
            && input_text == app.input
        {
            app.ghost_text = Some(prediction);
        }

        // Trigger prediction if needed (after 300ms debounce)
        if app.should_trigger_prediction() && app.ghost_text.is_none() && !app.input.is_empty() {
            let llm_client_clone = app.llm_client.clone();
            let input = app.input.clone();
            let tx = prediction_tx.clone();
            tokio::spawn(async move {
                if let Ok(prediction) = llm_client_clone.generate_input_completion(&input).await {
                    let _ = tx.send((input, prediction)).await;
                }
            });
            app.last_input_time = None; // Reset to prevent re-triggering
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    let telemetry = telemetry::TelemetryExporter::new(app.llm_client.usage_stats.clone());
    telemetry.export().await;

    Ok(())
}

fn check_key_match(key: &crossterm::event::KeyEvent, keybind_str: &str) -> bool {
    use crossterm::event::KeyModifiers;

    for combo in keybind_str.split(',') {
        let parts: Vec<&str> = combo.trim().split('+').collect();
        let mut target_modifiers = KeyModifiers::empty();
        let mut target_code = None;

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" => target_modifiers.insert(KeyModifiers::CONTROL),
                "alt" => target_modifiers.insert(KeyModifiers::ALT),
                "shift" => target_modifiers.insert(KeyModifiers::SHIFT),
                "return" | "enter" => target_code = Some(KeyCode::Enter),
                "backspace" => target_code = Some(KeyCode::Backspace),
                "delete" => target_code = Some(KeyCode::Delete),
                "esc" | "escape" => target_code = Some(KeyCode::Esc),
                "up" => target_code = Some(KeyCode::Up),
                "down" => target_code = Some(KeyCode::Down),
                "left" => target_code = Some(KeyCode::Left),
                "right" => target_code = Some(KeyCode::Right),
                c if c.len() == 1 => target_code = Some(KeyCode::Char(c.chars().next().unwrap())),
                _ => {}
            }
        }

        if let Some(code) = target_code
            && key.code == code
            && key.modifiers.contains(target_modifiers)
        {
            return true;
            }
        }
        false
    }

/// Parse agent spec (format: "provider:model" or just "provider")
fn parse_agent_spec(
    spec: &str,
) -> Result<(config::ProviderType, String), Box<dyn std::error::Error + Send + Sync>> {
    if spec.contains(':') {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        let provider = match parts[0].to_lowercase().as_str() {
            "ollama" => config::ProviderType::Ollama,
            "openrouter" => config::ProviderType::OpenRouter,
            "openai" => config::ProviderType::OpenAI,
            "gemini" => config::ProviderType::Gemini,
            "mistral" => config::ProviderType::Mistral,
            "anthropic" => config::ProviderType::Anthropic,
            _ => return Err(format!("Unknown provider: {}", parts[0]).into()),
        };
        Ok((provider, parts[1].to_string()))
    } else {
        let provider = match spec.to_lowercase().as_str() {
            "ollama" => config::ProviderType::Ollama,
            "openrouter" => config::ProviderType::OpenRouter,
            "openai" => config::ProviderType::OpenAI,
            "gemini" => config::ProviderType::Gemini,
            "mistral" => config::ProviderType::Mistral,
            "anthropic" => config::ProviderType::Anthropic,
            _ => return Err(format!("Unknown provider: {}", spec).into()),
        };
        let model = match provider {
            config::ProviderType::Ollama => "llama3".to_string(),
            config::ProviderType::OpenRouter => "openai/gpt-4".to_string(),
            config::ProviderType::OpenAI => "gpt-4".to_string(),
            config::ProviderType::Gemini => "gemini-pro".to_string(),
            config::ProviderType::Mistral => "mistral-large".to_string(),
            config::ProviderType::Anthropic => "claude-3-opus".to_string(),
        };
        Ok((provider, model))
    }
}

/// Run multiple agents in parallel and collect responses
async fn run_multi_agent(
    agent_specs: &[String],
    prompt: &str,
    base_config: &config::Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse agent specs
    let agents: Vec<(config::ProviderType, String)> = agent_specs
        .iter()
        .map(|spec| parse_agent_spec(spec))
        .collect::<Result<Vec<_>, _>>()?;

    if agents.is_empty() {
        return Ok(());
    }

    // Create shared managers
    let mcp_manager = Arc::new(Mutex::new(mcp::McpManager::new()));
    mcp_manager
        .lock()
        .await
        .load_from_config(&base_config.mcp)
        .await;

    let lsp_manager = Arc::new(Mutex::new(lsp::LspManager::new()));
    lsp_manager
        .lock()
        .await
        .load_from_config(&base_config.lsp)
        .await;

    let skill_manager = Arc::new(Mutex::new(skills::SkillManager::new()));
    {
        let mut skills = skill_manager.lock().await;
        skills.discover();
    }

    let custom_tool_manager = Arc::new(Mutex::new(custom_tools::CustomToolManager::new()));
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
            let llm_client = llm::LlmClient::new(config, mcp_mgr, lsp_mgr, skill_mgr, custom_mgr);

            let provider_name = format!("{:?}", provider);
            println!("Agent {} ({}) thinking...", provider_name, model);

            match llm_client.query_simple(&prompt).await {
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

/// Run OpenCrust in headless mode (no TUI, just prompt and response)
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
        config.provider = match provider_str.to_lowercase().as_str() {
            "ollama" => config::ProviderType::Ollama,
            "openrouter" => config::ProviderType::OpenRouter,
            "openai" => config::ProviderType::OpenAI,
            "gemini" => config::ProviderType::Gemini,
            "mistral" => config::ProviderType::Mistral,
            "anthropic" => config::ProviderType::Anthropic,
            _ => return Err(format!("Unknown provider: {}", provider_str).into()),
        };
    }

    // Override model if specified
    if let Some(model_str) = model {
        config.model = model_str.to_string();
    }

    // Create LLM client
    let llm_client = llm::LlmClient::new(
        config,
        mcp_manager,
        lsp_manager,
        skill_manager,
        custom_tool_manager,
    );

    // Send prompt and get response
    eprintln!("Sending prompt to LLM...");
    match llm_client.query_simple(&prompt_text).await {
        Ok(response) => {
            println!("{}", response);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
