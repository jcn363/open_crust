//! OpenCrust — Production TUI platform for AI-powered coding
//!
//! Entry point for the OpenCrust application. Parses CLI arguments via
//! clap, sets up shared subsystem managers (MCP, LSP, skills, custom tools),
//! dispatches to subcommand handlers or launches the interactive TUI.
//!
//! ## CLI Subcommands
//! - `acp` — JSON-RPC stdio mode for external process integration
//! - `run <command>` — Execute a shell command and exit
//! - `mcp` — MCP server management (list, install, browse, test, tools)
//! - `desktop` — Desktop integration (file picker, notifications, detection)
//! - `session` — Session management (list, show, delete, save, fork)
//! - `skills` — Skill management (list, activate, deactivate)
//! - `audit` — Audit log export, query, evidence, and compliance

#![deny(warnings)]
mod acp;
mod app;
mod audit;
mod clipboard;
mod compliance;
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
mod mcp_showcase;
mod mission_control;
mod models;
mod orchestrator;
mod permissions;
mod planner;
mod rag;
mod rules;
mod security;
mod sessions;
mod skills;
mod status_bar;
mod tool_executor;
mod tools;
mod ui;
mod web;

use desktop::detection::get_cinnamon_info;

use app::{App, Message, Mode};
use clap::{Parser, Subcommand};
use clipboard::ClipboardManager;
use crossterm::{
    ExecutableCommand,
    event::{Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mcp_showcase::McpShowcaseAction;
use mission_control::MissionControlAction;
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

    /// Override provider for this invocation (ollama, openrouter, openai, gemini, mistral, anthropic)
    #[arg(long, value_name = "PROVIDER")]
    provider: Option<String>,

    /// Override model for this invocation (default depends on provider; for openrouter default is openrouter/free-gpt-4o-mini, no API key required)
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
    /// Audit log management and compliance
    Audit {
        #[command(subcommand)]
        cmd: AuditCommands,
    },
}

#[derive(Subcommand, Debug)]
enum AuditCommands {
    /// Export audit logs to CSV or JSON
    Export {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        action: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value = "csv")]
        format: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// Query audit logs and display as table
    Query {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        action: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Build evidence package with SHA256 manifest
    Evidence {
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// Generate compliance report
    Report {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum McpCommands {
    /// List available MCP servers
    List,
    /// Install an MCP server by name
    Install { server: String },
    /// Launch MCP Showcase TUI browser (opens interactive TUI)
    Browse,
    /// Print MCP Showcase info to terminal (table of configured servers)
    Showcase,
    /// Test an MCP tool by calling it with arguments
    Test {
        /// Server name (e.g., "weather", "github")
        server: String,
        /// Tool name (e.g., "get_alerts", "get_forecast")
        tool: String,
        /// JSON arguments for the tool (e.g., '{"state": "CA"}')
        args: Option<String>,
    },
    /// List all tools across all MCP servers
    Tools,
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

    // Start background model list refresh if enabled
    if let Some(ref auto_refresh) = config.model_auto_refresh
        && auto_refresh.enabled
    {
        let refresh_config = config.clone();
        tokio::spawn(async move {
            let fetcher = models::ModelFetcher::new();
            let provider_str = match refresh_config.provider {
                config::ProviderType::Ollama => "ollama",
                config::ProviderType::OpenRouter => "openrouter",
                config::ProviderType::OpenAI => "openai",
                config::ProviderType::Gemini => "gemini",
                config::ProviderType::Mistral => "mistral",
                config::ProviderType::Anthropic => "anthropic",
                config::ProviderType::Groq => "groq",
                config::ProviderType::TogetherAi => "togetherai",
                config::ProviderType::Replicate => "replicate",
                config::ProviderType::DeepSeek => "deepseek",
                config::ProviderType::LocalAi => "localai",
            };
            // Fetch model list on startup (non-blocking background task)
            let models = fetcher.fetch(provider_str, None, None).await;
            if models.is_empty() {
                // Use bundled defaults as fallback
                let defaults = models::bundled_default_models();
                if defaults.contains_key(provider_str) {
                    eprintln!(
                        "[Models] Using bundled default model list for {}",
                        provider_str
                    );
                }
            } else {
                eprintln!(
                    "[Models] Refreshed {} model list ({} models)",
                    provider_str,
                    models.len()
                );
            }
        });
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
        Arc::new(config),
        mcp_manager.clone(),
        lsp_manager.clone(),
        skill_manager.clone(),
        custom_tool_manager.clone(),
    )?;

    if let Some(Commands::Acp) = args.command {
        return acp::run_acp_loop(llm_client).await;
    }

    if let Some(Commands::Run { command }) = args.command {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(output.status.code().unwrap_or(1));
        }
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
                println!("\nRestart opencrust to use the server.");
            }
            McpCommands::Browse => {
                println!("MCP Showcase TUI Browser");
                println!();
                println!("To launch the MCP Showcase TUI:");
                println!("  1. Run 'opencrust' without arguments to start the interactive TUI");
                println!("  2. Press Ctrl+M to open the MCP Showcase server browser");
                println!("  3. Navigate with arrow keys, toggle servers with Enter");
                println!("  4. Press Esc to return to the main chat");
                println!();
                println!("CLI alternatives:");
                println!("  'opencrust mcp showcase'  - Print server table to terminal");
                println!("  'opencrust mcp tools'     - List all tools");
                println!("  'opencrust mcp test <server> <tool> [args]' - Execute a tool");
            }
            McpCommands::Showcase => {
                let config = config::Config::load();
                println!("=== MCP Showcase ===");
                println!();
                if config.mcp.is_empty() {
                    println!("No MCP servers configured.");
                    println!("Use 'opencrust mcp list' to see available servers.");
                    println!("Use 'opencrust mcp install <name>' to install a server.");
                } else {
                    println!("{:<20} {:<15} {:<50}", "Name", "Status", "Command");
                    println!("{:<20} {:<15} {:<50}", "----", "------", "-------");
                    for (name, mcp_config) in &config.mcp {
                        let status = if mcp_config.enabled {
                            "Enabled"
                        } else {
                            "Disabled"
                        };
                        let cmd = mcp_config.command.join(" ");
                        let cmd_display = if cmd.len() > 47 {
                            format!("{}...", &cmd[..47])
                        } else {
                            cmd
                        };
                        println!("{:<20} {:<15} {:<50}", name, status, cmd_display);
                    }
                }
            }
            McpCommands::Test { server, tool, args } => {
                let config = config::Config::load();
                let mcp_manager = Arc::new(Mutex::new(mcp::McpManager::new()));
                mcp_manager.lock().await.load_from_config(&config.mcp).await;

                let arguments = match args {
                    Some(json_str) => serde_json::from_str(json_str.as_str())
                        .map_err(|e| format!("Invalid JSON arguments: {}", e)),
                    None => Ok(serde_json::json!({})),
                };

                match arguments {
                    Ok(args_val) => {
                        let full_name = format!("{}_{}", server, tool);
                        println!("Calling MCP tool '{}' on server '{}'...", tool, server);
                        println!(
                            "Arguments: {}",
                            serde_json::to_string_pretty(&args_val).unwrap_or_default()
                        );
                        println!();
                        match mcp_manager
                            .lock()
                            .await
                            .call_tool(&full_name, &args_val)
                            .await
                        {
                            Ok(result) => {
                                println!("=== Result ===");
                                println!("{}", result);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
            McpCommands::Tools => {
                let config = config::Config::load();
                let mcp_manager = Arc::new(Mutex::new(mcp::McpManager::new()));
                mcp_manager.lock().await.load_from_config(&config.mcp).await;

                println!("=== MCP Tools ===");
                println!();

                let tools = mcp_manager.lock().await.list_tools().await;
                if tools.is_empty() {
                    println!(
                        "No tools found. Make sure you have MCP servers configured and enabled."
                    );
                    println!("Use 'opencrust mcp list' to see available servers.");
                    println!("Use 'opencrust mcp install <name>' to install a server.");
                } else {
                    println!("Found {} tools:", tools.len());
                    println!();
                    for tool in &tools {
                        let name = tool
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let desc = tool
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("No description");
                        println!("  {} {}", name, desc);
                        if let Some(schema) = tool.get("inputSchema")
                            && let Some(props) = schema.get("properties")
                            && let Some(props_obj) = props.as_object()
                            && !props_obj.is_empty()
                        {
                            println!("    Arguments:");
                            for (prop_name, prop_info) in props_obj {
                                let prop_type = prop_info
                                    .get("type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("any");
                                let prop_desc = prop_info
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if prop_desc.is_empty() {
                                    println!("      - {}: {}", prop_name, prop_type);
                                } else {
                                    println!(
                                        "      - {} ({}): {}",
                                        prop_name, prop_type, prop_desc
                                    );
                                }
                            }
                        }
                        println!();
                    }
                }
            }
        }
        return Ok(());
    }

    // Desktop integration commands
    if let Some(Commands::Desktop { cmd }) = &args.command {
        match cmd {
            DesktopCommands::FilePicker { mode, dir, title } => {
                use desktop::file_picker::{
                    FilePickerMode, FilePickerOptions, detect_file_picker_backend, file_picker,
                    is_file_picker_available,
                };

                if !is_file_picker_available() {
                    eprintln!(
                        "Error: No file picker backend available (need nemo, zenity, or kdialog)"
                    );
                    return Ok(());
                }
                let backend = detect_file_picker_backend();

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
                println!("Backend: {}", backend.name());
            }
            DesktopCommands::Notify {
                title,
                body,
                urgency,
            } => {
                use desktop::notifications::{
                    Notification, NotificationUrgency, is_notification_available, notify_error,
                    notify_success, send_notification_smart,
                };

                if !is_notification_available() {
                    eprintln!("Warning: No notification daemon available");
                }

                let urgency = NotificationUrgency::from_str(urgency);
                let notification = Notification::new(title, body)
                    .with_urgency(urgency)
                    .with_expire_timeout(10);

                match send_notification_smart(&notification) {
                    Ok(_) => {
                        println!("Notification sent: {} - {}", title, body);
                        let _ =
                            notify_success("Notification sent", format!("{} - {}", title, body));
                    }
                    Err(e) => {
                        eprintln!("Failed to send notification: {}", e);
                        let _ = notify_error("Notification failed", e.to_string());
                    }
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
    // Audit and compliance commands
    if let Some(Commands::Audit { cmd }) = &args.command {
        let config = config::Config::load();
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("opencrust/logs");
        let audit_log_path = log_dir.join("audit.log");

        match cmd {
            AuditCommands::Export {
                from,
                to,
                action,
                status,
                format,
                output,
            } => {
                let from_date = from
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
                let to_date = to
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
                let status_filter = status.as_ref().map(|s| s == "approved");

                let query = audit::AuditQuery {
                    from_date,
                    to_date,
                    action_pattern: action.clone(),
                    status_filter,
                };

                match query.execute(&audit_log_path) {
                    Ok(entries) => {
                        let fmt = match format.as_str() {
                            "json" => audit::ExportFormat::Json,
                            _ => audit::ExportFormat::Csv,
                        };

                        match output {
                            Some(path) => {
                                let out_path = std::path::Path::new(&path);
                                audit::AuditExport::export_to_file(&entries, fmt, out_path)
                                    .unwrap_or_else(|e| eprintln!("Export error: {}", e));
                                println!("Exported {} entries to {}", entries.len(), path);
                            }
                            None => {
                                audit::AuditExport::export(&entries, fmt, &mut std::io::stdout())
                                    .unwrap_or_else(|e| eprintln!("Export error: {}", e));
                            }
                        }
                    }
                    Err(e) => eprintln!("Error querying audit log: {}", e),
                }
            }
            AuditCommands::Query {
                from,
                to,
                action,
                status,
            } => {
                let from_date = from
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
                let to_date = to
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
                let status_filter = status.as_ref().map(|s| s == "approved");

                let query = audit::AuditQuery {
                    from_date,
                    to_date,
                    action_pattern: action.clone(),
                    status_filter,
                };

                match query.execute(&audit_log_path) {
                    Ok(entries) => {
                        if entries.is_empty() {
                            println!("No matching audit entries found.");
                        } else {
                            println!(
                                "{:<26} {:<16} {:<10} {:<20} {:<8}",
                                "Timestamp", "Session", "Agent", "Tool", "Status"
                            );
                            println!("{}", "-".repeat(80));
                            for entry in &entries {
                                let status_str = if entry.approved { "APPROVED" } else { "DENIED" };
                                println!(
                                    "{:<26} {:<16} {:<10} {:<20} {:<8}",
                                    entry.timestamp,
                                    entry.session_id.chars().take(14).collect::<String>(),
                                    entry.agent_type.chars().take(8).collect::<String>(),
                                    entry.tool.chars().take(18).collect::<String>(),
                                    status_str,
                                );
                            }
                            println!("\nTotal: {} entries", entries.len());
                        }
                    }
                    Err(e) => eprintln!("Error querying audit log: {}", e),
                }
            }
            AuditCommands::Evidence { output_dir } => {
                let out_dir = output_dir
                    .clone()
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                match compliance::EvidencePackage::build(&audit_log_path, &config, &out_dir) {
                    Ok(()) => {}
                    Err(e) => eprintln!("Error building evidence package: {}", e),
                }
            }
            AuditCommands::Report { from, to } => {
                let from_date = from
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
                let to_date = to
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

                let query = audit::AuditQuery {
                    from_date,
                    to_date,
                    action_pattern: None,
                    status_filter: None,
                };

                match query.execute(&audit_log_path) {
                    Ok(entries) => {
                        let report = compliance::ComplianceReport::generate(&entries);
                        println!("{}", report);
                    }
                    Err(e) => eprintln!("Error generating report: {}", e),
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
                for (name, description, active) in skills.list_skills_with_stats() {
                    let status = if active { "ACTIVE" } else { "inactive" };
                    println!("[{}] {} - {}", status, name, description);
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
        }
        return Ok(());
    }

    let (prompt_tx, mut prompt_rx) = mpsc::channel::<String>(32);
    let (response_tx, mut response_rx) = mpsc::channel::<String>(32);
    let (approval_tx, mut approval_rx) = mpsc::channel::<bool>(1);
    let (background_task_tx, mut background_task_rx) = mpsc::channel::<String>(32);
    let (prediction_tx, mut prediction_rx) = mpsc::channel::<(String, String)>(32); // (input_text, prediction)

    let client_clone = llm_client.clone();
    tokio::spawn(async move {
        let mut messages_history: Vec<Value> = Vec::new();
        while let Some(prompt) = prompt_rx.recv().await {
            let prompt_str = prompt.trim();

            if prompt_str == "/init" {
                match rules::init_project_rules() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("opencrust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/provider ") {
                let new_provider = prompt_str.trim_start_matches("/provider ").trim();
                let mut new_config = (*client_clone.config).clone();
                match new_provider.to_lowercase().as_str() {
                    "ollama" => {
                        new_config.provider = config::ProviderType::Ollama;
                        new_config.save();
                        let _ = response_tx
                            .send("opencrust: Provider switched to Ollama".to_string())
                            .await;
                    }
                    "openrouter" => {
                        new_config.provider = config::ProviderType::OpenRouter;
                        new_config.save();
                        let _ = response_tx
                            .send("opencrust: Provider switched to OpenRouter".to_string())
                            .await;
                    }
                    _ => {
                        let _ = response_tx
                            .send(format!("opencrust: Unknown provider '{}'", new_provider))
                            .await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/model ") {
                let new_model = prompt_str.trim_start_matches("/model ").trim();
                let mut new_config = (*client_clone.config).clone();
                new_config.model = new_model.to_string();
                new_config.save();
                let _ = response_tx
                    .send(format!("opencrust: Model switched to '{}'", new_model))
                    .await;
                continue;
            } else if prompt_str == "/undo" {
                match git::undo() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("opencrust: {}", msg)).await;
                    }
                    Err(e) => {
                        let _ = response_tx.send(format!("Error: {}", e)).await;
                    }
                }
                continue;
            } else if prompt_str == "/redo" {
                match git::redo() {
                    Ok(msg) => {
                        let _ = response_tx.send(format!("opencrust: {}", msg)).await;
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
                .send(String::from("opencrust: Thinking..."))
                .await;

            let res = client_clone
                .send_message(
                    &mut messages_history,
                    &enriched_prompt,
                    response_tx.clone(),
                    Some(&mut approval_rx),
                )
                .await;
            match res {
                Ok(reply) => {
                    let _ = response_tx.send(format!("opencrust: {}", reply)).await;
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
        (*llm_client.config).clone(),
        prompt_tx,
        approval_tx,
        background_task_tx,
        llm_client.clone(),
    );
    app.refresh_sidebar();

    // Frame rate limiting for smoother UI
    let mut last_frame = std::time::Instant::now();
    let target_frame_duration = std::time::Duration::from_millis(16); // ~60 FPS

    // Populate skill browser items from skill manager
    {
        let skills = skill_manager.lock().await;
        let skill_list = skills.list_skills_with_stats();
        for (name, description, active) in skill_list {
            app.skill_browser_items.push((name, description, active));
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
                    .map(|s| s.splitn(3, '|').collect())
                    .unwrap_or_default();
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

            let tab_idx = app.active_tab.min(app.tabs.len().saturating_sub(1));
            let tab = &mut app.tabs[tab_idx];
            if let Some(last) = tab.messages.last()
                && (last.content == "opencrust: Thinking..."
                    || last.content.starts_with("opencrust: Executing tool"))
            {
                tab.messages.pop();
            }
            tab.messages.push(Message::new(response));
            // Auto-scroll to bottom on new messages
            app.message_scroll = 0;
        }

        // Handle background task notifications
        while let Ok(notification) = background_task_rx.try_recv() {
            app.handle_background_notification(&notification);
        }

        // Periodic skill hot-reload check
        {
            let mut skills = skill_manager.lock().await;
            if skills.should_check_for_updates() {
                let (added, removed, modified) = skills.discover_changes();
                if !added.is_empty() {
                    eprintln!("[Skills] Discovered new skills: {}", added.join(", "));
                    for name in &added {
                        if let Some(skill) = skills.get_skill(name) {
                            app.skill_browser_items.push((
                                skill.metadata.name.clone(),
                                skill.metadata.description.clone(),
                                skill.active,
                            ));
                        }
                    }
                }
                if !removed.is_empty() {
                    eprintln!("[Skills] Removed skills: {}", removed.join(", "));
                    app.skill_browser_items
                        .retain(|(name, _, _)| !removed.contains(name));
                }
                if !modified.is_empty() {
                    eprintln!("[Skills] Modified skills: {}", modified.join(", "));
                    for name in &modified {
                        if let Some(skill) = skills.get_skill(name) {
                            // Remove stale entry first, then re-add with updated data
                            app.skill_browser_items.retain(|(n, _, _)| n != name);
                            app.skill_browser_items.push((
                                skill.metadata.name.clone(),
                                skill.metadata.description.clone(),
                                skill.active,
                            ));
                        }
                    }
                }
            }
        }

        // Frame rate limiting for smoother UI
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_frame);
        if elapsed < target_frame_duration {
            tokio::time::sleep(target_frame_duration - elapsed).await;
        }
        last_frame = std::time::Instant::now();
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Some(Event::Key(key)) = events::next_event().await? {
            // Check for Copy (Ctrl+C) - copy current input to clipboard
            if check_key_match(&key, &copy_key) {
                if !app.input.is_empty() && clipboard.copy(&app.input) {
                    app.tabs[0]
                        .messages
                        .push(Message::new(String::from("Copied to clipboard")));
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
                        app.tabs[0]
                            .messages
                            .push(Message::new(String::from("You: y (Approved)")));
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.waiting_for_approval = false;
                        if let Some(tx) = &app.approval_tx {
                            let _ = tx.try_send(false);
                        }
                        app.tabs[0]
                            .messages
                            .push(Message::new(String::from("You: n (Denied)")));
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
                        KeyCode::Up => {
                            // Scroll message list up
                            let tab = &app.tabs[app.active_tab];
                            let msg_count = tab.messages.len()
                                + if app.active_tab == 1 {
                                    app.background_tasks.len()
                                } else {
                                    0
                                };
                            if app.message_scroll < msg_count.saturating_sub(1) {
                                app.message_scroll += 1;
                            }
                        }
                        KeyCode::Down => {
                            // Scroll message list down
                            app.message_scroll = app.message_scroll.saturating_sub(1);
                        }
                        KeyCode::PageUp => {
                            // Scroll up by 10 lines
                            let tab = &app.tabs[app.active_tab];
                            let msg_count = tab.messages.len()
                                + if app.active_tab == 1 {
                                    app.background_tasks.len()
                                } else {
                                    0
                                };
                            app.message_scroll = app
                                .message_scroll
                                .saturating_add(10)
                                .min(msg_count.saturating_sub(1));
                        }
                        KeyCode::PageDown => {
                            // Scroll down by 10 lines
                            app.message_scroll = app.message_scroll.saturating_sub(10);
                        }
                        KeyCode::Home => {
                            // Scroll to the very top (clamped in draw_message_list)
                            app.message_scroll = usize::MAX;
                        }
                        KeyCode::End => {
                            app.message_scroll = 0;
                        }
                        KeyCode::Char('?') => {
                            app.mode = Mode::Help;
                        }
                        // Sidebar navigation: [ = up, ] = down
                        KeyCode::Char('[')
                            if app.show_sidebar
                                && app.sidebar_selected > 0
                                && !app.sidebar_items.is_empty() =>
                        {
                            app.sidebar_selected -= 1;
                        }
                        KeyCode::Char(']')
                            if app.show_sidebar
                                && app.sidebar_selected + 1 < app.sidebar_items.len()
                                && !app.sidebar_items.is_empty() =>
                        {
                            app.sidebar_selected += 1;
                        }
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
                                .push(Message::new(String::from("Entering plan mode...")));
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
                                app.tabs[0].messages.push(Message::new(String::from(
                                    "Spawning background task...",
                                )));
                            } else {
                                app.tabs[0].messages.push(Message::new(String::from(
                                    "No input to spawn as background task",
                                )));
                            }
                        }
                        KeyCode::Tab => {
                            app.active_tab = (app.active_tab + 1) % app.tabs.len();
                        }
                        KeyCode::Char('v')
                            if key.modifiers == crossterm::event::KeyModifiers::ALT =>
                        {
                            app.vim_mode = !app.vim_mode;
                            let mode_str = if app.vim_mode { "enabled" } else { "disabled" };
                            app.tabs[0]
                                .messages
                                .push(Message::new(format!("Vim Mode {}", mode_str)));
                        }
                        KeyCode::Char('m')
                            if key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                        {
                            // Build server list from config
                            let servers: Vec<crate::mcp_showcase::McpServerInfo> = app
                                .config
                                .mcp
                                .iter()
                                .map(|(name, mcp_config)| crate::mcp_showcase::McpServerInfo {
                                    name: name.clone(),
                                    description: crate::mcp_showcase::get_server_description(name),
                                    installed: true,
                                    enabled: mcp_config.enabled,
                                })
                                .collect();
                            app.mode = Mode::McpShowcase;
                            app.mcp_showcase_ui =
                                Some(crate::mcp_showcase::McpShowcaseUI::new(servers));
                        }
                        KeyCode::Char('g')
                            if key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                        {
                            app.mode = Mode::MissionControl;
                            if app.mission_control_ui.is_none() {
                                app.mission_control_ui =
                                    Some(crate::mission_control::MissionControlUI::new());
                            }
                            // Refresh tasks from orchestrator bridge
                            if let Some(ref tasks_arc) = app.orchestrator_tasks
                                && let Some(ref mut ui) = app.mission_control_ui
                            {
                                ui.refresh_tasks(Some(tasks_arc));
                            }
                        }
                        _ => {}
                    },
                    Mode::Insert => {
                        if check_key_match(&key, &submit_keys) {
                            app.submit_message();
                        } else if app.vim_mode {
                            // Vim Mode input editing
                            match key.code {
                                KeyCode::Esc => {
                                    if app.ghost_text.is_some() {
                                        app.clear_ghost_text();
                                    } else {
                                        app.enter_normal_mode();
                                    }
                                }
                                KeyCode::Backspace => {
                                    app.handle_backspace();
                                }
                                // Vim navigation (specific chars BEFORE general Char(c))
                                KeyCode::Char('h') => {
                                    app.move_cursor_left();
                                }
                                KeyCode::Char('l') => {
                                    app.move_cursor_right();
                                }
                                KeyCode::Char('w') => {
                                    app.move_to_next_word();
                                }
                                KeyCode::Char('b') => {
                                    app.move_to_prev_word();
                                }
                                KeyCode::Char('0') => {
                                    app.move_to_line_start();
                                }
                                KeyCode::Char('$') => {
                                    app.move_to_line_end();
                                }
                                KeyCode::Char('a') => {
                                    app.move_cursor_right();
                                }
                                KeyCode::Char('d') => {
                                    app.delete_line();
                                }
                                KeyCode::Char('c') => {
                                    app.delete_line();
                                }
                                KeyCode::Char('y') => {
                                    let _ = app.yank_line(&mut clipboard);
                                }
                                KeyCode::Char(c) => {
                                    app.handle_char(c);
                                    if app.input_prediction_enabled {
                                        app.last_input_time = Some(std::time::Instant::now());
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Tab => {
                                    if let Some(ghost) = app.ghost_text.take() {
                                        app.input.push_str(&ghost);
                                        app.last_input_time = Some(std::time::Instant::now());
                                    }
                                }
                                KeyCode::Esc => {
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
                                    if app.input_prediction_enabled {
                                        app.last_input_time = Some(std::time::Instant::now());
                                    }
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
                                    app.tabs[0].messages.push(Message::new(format!(
                                        "Error writing {}: {}",
                                        change.path, e
                                    )));
                                } else {
                                    app.tabs[0]
                                        .messages
                                        .push(Message::new(format!("Applied: {}", change.path)));
                                }
                            }

                            // Clear reviewed changes
                            app.proposed_changes.clear();
                            app.plan_review_index = 0;
                            app.mode = Mode::Normal;
                            app.tabs[0].messages.push(Message::new(format!(
                                "Executed {} approved changes",
                                approved.len()
                            )));
                        }
                        // Cancel (Esc)
                        KeyCode::Esc => {
                            app.proposed_changes.clear();
                            app.plan_review_index = 0;
                            app.mode = Mode::Normal;
                            app.tabs[0]
                                .messages
                                .push(Message::new(String::from("Plan cancelled")));
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
                                app.tabs[0].messages.push(Message::new(format!("System: Installed MCP server '{}'. Restart opencrust to use it.", name)));
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
                            if let Some((name, _, active)) =
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
                                app.tabs[0].messages.push(Message::new(format!(
                                    "System: Skill '{}' {}",
                                    name, status
                                )));
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
                                        config::ProviderType::Groq,
                                        config::ProviderType::TogetherAi,
                                        config::ProviderType::Replicate,
                                        config::ProviderType::DeepSeek,
                                        config::ProviderType::LocalAi,
                                    ];
                                    let current_idx = providers
                                        .iter()
                                        .position(|p| p == &app.config.provider)
                                        .unwrap_or(0);
                                    let next_idx = (current_idx + 1) % providers.len();
                                    app.config.provider = providers[next_idx].clone();
                                    app.config.save();
                                    app.tabs[0].messages.push(Message::new(format!(
                                        "Provider switched to {:?}",
                                        app.config.provider
                                    )));
                                    app.mode = Mode::Normal;
                                }
                                1 => {
                                    // Switch Model
                                    // For simplicity, just show a message - in a real implementation this would open a model selector
                                    app.tabs[0].messages.push(Message::new(format!("Model switching not fully implemented yet. Current model: {}", app.config.model)));
                                    app.mode = Mode::Normal;
                                }
                                2 => {
                                    // Clear Context
                                    app.tabs[0].messages.clear();
                                    app.tabs[0].messages.push(Message::new(String::from("Welcome to opencrust. Press 'i' to enter insert mode, 's' for servers, 'q' to quit.")));
                                    app.history.clear();
                                    app.save_history();
                                    app.tabs[0]
                                        .messages
                                        .push(Message::new("Context cleared.".to_string()));
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
                    Mode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.mode = Mode::Normal;
                        }
                        _ => {}
                    },
                    Mode::McpShowcase => {
                        if let Some(ref mut ui) = app.mcp_showcase_ui {
                            match ui.handle_key(key.code) {
                                McpShowcaseAction::ToggleServer(name) => {
                                    // Toggle enabled status in config
                                    if let Some(server_cfg) = app.config.mcp.get_mut(&name) {
                                        server_cfg.enabled = !server_cfg.enabled;
                                        // Save updated config to disk
                                        app.config.save();
                                        // Update UI server list to reflect change
                                        ui.toggle_server(&name);
                                    }
                                }
                                McpShowcaseAction::ExitMode => {
                                    app.mode = Mode::Normal;
                                }
                                McpShowcaseAction::None => {}
                            }
                        }
                    }
                    Mode::MissionControl => {
                        if let Some(ref mut ui) = app.mission_control_ui {
                            // Refresh tasks from orchestrator bridge before handling input
                            if let Some(ref tasks_arc) = app.orchestrator_tasks {
                                ui.refresh_tasks(Some(tasks_arc));
                            }
                            if let MissionControlAction::ExitMode = ui.handle_key(key.code) {
                                app.mode = Mode::Normal;
                            }
                        }
                    }
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
                c if c.len() == 1 => target_code = c.chars().next().map(KeyCode::Char),
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
            config::ProviderType::Ollama => "deepseek-r1".to_string(),
            config::ProviderType::OpenRouter => "openrouter/free-gpt-4o-mini".to_string(),
            config::ProviderType::OpenAI => "gpt-4o-mini".to_string(),
            config::ProviderType::Gemini => "gemini-2.0-flash".to_string(),
            config::ProviderType::Mistral => "mistral-small".to_string(),
            config::ProviderType::Anthropic => "claude-sonnet-4-20250514".to_string(),
            _ => "deepseek-r1".to_string(),
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
            let provider_name = format!("{:?}", provider);
            let llm_client = match llm::LlmClient::new(
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
