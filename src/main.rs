//! OpenCrust — Production TUI platform for AI-powered coding
//!
//! Entry point for the OpenCrust application. Parses CLI arguments via
//! clap, sets up shared subsystem managers (MCP, LSP, skills, custom tools,
//! plugins, background agents, multi-repo), dispatches to subcommand
//! handlers or launches the interactive TUI.
//!
//! ## CLI Subcommands
//! - `acp` — JSON-RPC stdio mode for external process integration
//! - `run <command>` — Execute a shell command and exit
//! - `mcp` — MCP server management (list, install, browse, test, tools)
//! - `desktop` — Desktop integration (file picker, notifications, detection)
//! - `session` — Session management (list, show, delete, save, fork)
//! - `skills` — Skill management (list, activate, deactivate)
//! - `audit` — Audit log export, query, evidence, and compliance
//! - `background` — Background agent management and dashboard
//! - `plugin` — Plugin/extension management (list, install, remove, enable, disable)
//! - `repo` — Multi-repository management (add, remove, list, search, git)

#![deny(warnings)]
mod acp;
mod app;
mod audit;
mod background_agents;
mod cli;
mod clipboard;
mod compliance;
mod config;
mod context;
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
mod rag;
mod rules;
mod security;
mod sessions;
mod skills;
mod startup;
mod status_bar;
mod tool_executor;
mod tools;
mod ui;
mod web;

use clap::Parser;
use cli::*;
use event_loop::run_tui;
use startup::*;

use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

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
                    Ok(path) => println!("Evidence package created at: {}", path.display()),
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
            AuditCommands::Policy { output_dir } => {
                let out_dir = output_dir
                    .clone()
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                let compliance_mgr = compliance::ComplianceManager::new(&config);
                match compliance_mgr.full_check(&out_dir) {
                    Ok(report) => {
                        println!("{}", report);
                        if !report.violations.is_empty() {
                            println!("\nPolicy violations found: {}", report.violations.len());
                            for v in &report.violations {
                                println!("  [{}] {}: {}",
                                    v.severity, v.rule_id, v.message);
                            }
                        } else {
                            println!("\nNo policy violations found.");
                        }
                    }
                    Err(e) => eprintln!("Error running compliance check: {}", e),
                }
            }
            AuditCommands::Verify { path } => {
                let pkg_dir = std::path::PathBuf::from(&path);
                if !pkg_dir.exists() {
                    eprintln!("Error: evidence package directory '{}' not found", path);
                    return Ok(());
                }
                match compliance::EvidencePackage::verify(&pkg_dir) {
                    Ok(results) => {
                        println!("Verification results for: {}", pkg_dir.display());
                        let mut all_valid = true;
                        for (name, valid, hash) in &results {
                            let status = if *valid { "VALID" } else { "MISMATCH" };
                            if !*valid { all_valid = false; }
                            println!("  {}: {} ({})", name, status, hash);
                        }
                        if all_valid {
                            println!("\n✓ All files verified successfully.");
                        } else {
                            println!("\n✗ Some files failed verification!");
                        }
                    }
                    Err(e) => eprintln!("Error verifying evidence package: {}", e),
                }
            }
            AuditCommands::Check { output_dir } => {
                let out_dir = std::path::PathBuf::from(&output_dir);
                let compliance_mgr = compliance::ComplianceManager::new(&config);
                match compliance_mgr.full_check(&out_dir) {
                    Ok(report) => {
                        println!("=== Full Compliance Check ===");
                        println!("{}", report);
                        if report.violations.is_empty() {
                            println!("\n✓ All compliance checks passed.");
                        } else {
                            println!("\n✗ {} policy violation(s) found.", report.violations.len());
                        }
                        // Also build evidence package
                        match compliance::EvidencePackage::build(
                            &audit_log_path, &config, &out_dir
                        ) {
                            Ok(path) => println!("\nEvidence package: {}", path.display()),
                            Err(e) => eprintln!("\nWarning: evidence package build failed: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Error running compliance check: {}", e),
                }
            }
        }
        return Ok(());
    }

    // Background agent management commands
    if let Some(Commands::Background { cmd }) = &args.command {
        let agent_mgr = background_agents::BackgroundAgentManager::new();
        let config = config::Config::load();
        match cmd {
            BackgroundCommands::List => {
                let agents = agent_mgr.list().await;
                if agents.is_empty() {
                    println!("No background agents.");
                } else {
                    println!("{:<5} {:<24} {:<10} {:<12} {}", "ID", "Name", "Status", "Progress", "Age");
                    println!("{}", "-".repeat(90));
                    for agent in &agents {
                        let id_short = agent.id.to_string().chars().take(8).collect::<String>();
                        let status_str = match &agent.status {
                            background_agents::AgentStatus::Pending => "PENDING",
                            background_agents::AgentStatus::Running => "RUNNING",
                            background_agents::AgentStatus::Completed { .. } => "DONE",
                            background_agents::AgentStatus::Failed { .. } => "FAILED",
                            background_agents::AgentStatus::Cancelled => "CANCELLED",
                        };
                        let age = (chrono::Utc::now() - agent.created_at).num_seconds();
                        println!(
                            "{:<5} {:<24} {:<10} {:>3}%    {}s",
                            id_short, agent.name, status_str, agent.progress, age
                        );
                    }
                }
                // Also print stats
                let stats = agent_mgr.stats().await;
                println!("\n{}", stats);
            }
            BackgroundCommands::Show { id } => {
                let uid = match uuid::Uuid::parse_str(id) {
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
                        if let background_agents::AgentStatus::Completed { output } = &agent.status {
                            println!("\n  Result (first 500 chars):");
                            println!("    {}", &output.chars().take(500).collect::<String>());
                        }
                        if let background_agents::AgentStatus::Failed { error } = &agent.status {
                            println!("\n  Error: {}", error);
                        }
                    }
                    None => eprintln!("Agent '{}' not found.", id),
                }
            }
            BackgroundCommands::Start { name, prompt } => {
                // Build LlmClient for the agent
                let (mcp_mgr, lsp_mgr, skill_mgr, custom_tool_mgr) = init_managers(&config).await;
                let llm_client = match llm::LlmClient::new(
                    std::sync::Arc::new(config),
                    mcp_mgr, lsp_mgr, skill_mgr, custom_tool_mgr,
                ) {
                    Ok(c) => std::sync::Arc::new(c),
                    Err(e) => {
                        eprintln!("Error creating LLM client: {}", e);
                        return Ok(());
                    }
                };
                let agent_id = agent_mgr.spawn(
                    name.clone(),
                    prompt.clone(),
                    None, None, vec![],
                    llm_client,
                ).await;
                println!("Started background agent '{}' with ID: {}", name, agent_id);
            }
            BackgroundCommands::Cancel { id } => {
                let uid = match uuid::Uuid::parse_str(id) {
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
                let uid = match uuid::Uuid::parse_str(id) {
                    Ok(u) => u,
                    Err(_) => {
                        eprintln!("Invalid UUID: {}", id);
                        return Ok(());
                    }
                };
                match agent_mgr.get(&uid).await {
                    Some(agent) => {
                        let start = agent.log.len().saturating_sub(*lines);
                        for line in agent.log.iter().skip(start) {
                            println!("{}", line);
                        }
                    }
                    None => eprintln!("Agent '{}' not found.", id),
                }
            }
        }
        return Ok(());
    }

    // Plugin management commands
    if let Some(Commands::Plugin { cmd }) = &args.command {
        let mut plugin_mgr = plugins::PluginManager::new();
        plugin_mgr.discover();
        match cmd {
            PluginCommands::List => {
                let plugins = plugin_mgr.list();
                if plugins.is_empty() {
                    println!("No plugins discovered.");
                    println!("\nSearch paths:");
                    println!("  - .opencrust/plugins/");
                    if let Some(config_dir) = dirs::config_dir() {
                        println!("  - {}", config_dir.join("opencrust/plugins").display());
                    }
                } else {
                    println!("{:<24} {:<10} {:<8} {:<40}", "Name", "Version", "Status", "Description");
                    println!("{}", "-".repeat(90));
                    for p in &plugins {
                        let status = if p.enabled { "enabled" } else { "disabled" };
                        let desc = if p.description.len() > 38 {
                            format!("{}...", &p.description[..35])
                        } else {
                            p.description.clone()
                        };
                        println!("{:<24} {:<10} {:<8} {:<40}", p.name, p.version, status, desc);
                    }
                }
            }
            PluginCommands::Show { name } => {
                match plugin_mgr.get(name) {
                    Some(p) => {
                        println!("Name:        {}", p.name);
                        println!("Version:     {}", p.version);
                        println!("Description: {}", p.description);
                        println!("Author:      {}", p.author);
                        println!("Path:        {}", p.path.display());
                        println!("Enabled:     {}", p.enabled);
                        println!("Entry:       {}", p.entry.as_deref().unwrap_or("(none)"));
                        println!("Hooks:       {}", p.hooks.join(", "));
                        println!("Tools:       {}", p.tools.join(", "));
                        println!("Deps:        {}", p.dependencies.join(", "));
                    }
                    None => eprintln!("Plugin '{}' not found.", name),
                }
            }
            PluginCommands::Install { path } => {
                let src = std::path::PathBuf::from(&path);
                if !src.exists() {
                    eprintln!("Error: path '{}' does not exist", path);
                    return Ok(());
                }
                match plugin_mgr.install(&src) {
                    Ok(name) => println!("Plugin '{}' installed successfully.", name),
                    Err(e) => eprintln!("Error installing plugin: {}", e),
                }
            }
            PluginCommands::Remove { name } => {
                match plugin_mgr.remove(&name) {
                    Ok(_) => println!("Plugin '{}' removed.", name),
                    Err(e) => eprintln!("Error removing plugin: {}", e),
                }
            }
            PluginCommands::Enable { name } => {
                match plugin_mgr.enable(&name) {
                    Ok(_) => println!("Plugin '{}' enabled.", name),
                    Err(e) => eprintln!("Error enabling plugin: {}", e),
                }
            }
            PluginCommands::Disable { name } => {
                match plugin_mgr.disable(&name) {
                    Ok(_) => println!("Plugin '{}' disabled.", name),
                    Err(e) => eprintln!("Error disabling plugin: {}", e),
                }
            }
            PluginCommands::Stats => {
                let stats = plugin_mgr.stats();
                println!("{}", stats);
            }
        }
        return Ok(());
    }

    // Multi-repo management commands
    if let Some(Commands::Repo { cmd }) = &args.command {
        let repo_mgr = multi_repo::MultiRepoManager::new();
        match cmd {
            RepoCommands::List => {
                let repos = repo_mgr.list().await;
                if repos.is_empty() {
                    println!("No repositories registered.");
                    println!("\nUse 'opencrust repo add <name> <path>' to register one.");
                } else {
                    println!("{:<20} {:<20} {:<30} {}", "Name", "Branch", "Path", "Remote");
                    println!("{}", "-".repeat(100));
                    for repo in &repos {
                        let branch = repo.branch.as_deref().unwrap_or("(detached)");
                        let remote = repo.remote.as_deref().unwrap_or("-");
                        let path_str = repo.path.display().to_string();
                        let path_short = if path_str.len() > 28 {
                            format!("...{}", &path_str[path_str.len().saturating_sub(25)..])
                        } else {
                            path_str
                        };
                        println!("{:<20} {:<20} {:<30} {}", repo.name, branch, path_short, remote);
                    }
                }
            }
            RepoCommands::Show { name } => {
                match repo_mgr.get(name).await {
                    Some(repo) => {
                        println!("Name:       {}", repo.name);
                        println!("Path:       {}", repo.path.display());
                        println!("Branch:     {}", repo.branch.as_deref().unwrap_or("(detached)"));
                        println!("Remote:     {}", repo.remote.as_deref().unwrap_or("(none)"));
                        println!("Tags:       {}", repo.tags.join(", "));
                        println!("Registered: {}", repo.registered_at);
                        if let Some(idx) = repo.last_indexed {
                            println!("Indexed:    {}", idx);
                        }
                    }
                    None => eprintln!("Repository '{}' not found.", name),
                }
            }
            RepoCommands::Add { name, path, tags } => {
                let tags: Vec<String> = tags
                    .as_ref()
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();
                match repo_mgr.add(name.clone(), std::path::PathBuf::from(&path), tags).await {
                    Ok(repo) => {
                        println!("Repository '{}' registered at {}", repo.name, repo.path.display());
                        println!("  Branch: {}", repo.branch.as_deref().unwrap_or("(detached)"));
                        if let Some(remote) = &repo.remote {
                            println!("  Remote: {}", remote);
                        }
                    }
                    Err(e) => eprintln!("Error adding repository: {}", e),
                }
            }
            RepoCommands::Remove { name } => {
                if repo_mgr.remove(&name).await {
                    println!("Repository '{}' removed.", name);
                } else {
                    eprintln!("Repository '{}' not found.", name);
                }
            }
            RepoCommands::Stats => {
                let stats = repo_mgr.stats().await;
                let repos = repo_mgr.list().await;
                println!("{}", stats);
                for repo in &repos {
                    println!("  {}", repo.summary());
                }
            }
            RepoCommands::Git { args } => {
                let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let results = repo_mgr.git_command_all(&args_refs).await;
                if results.is_empty() {
                    println!("No repositories to run command on.");
                } else {
                    for (repo, result) in &results {
                        println!("\n=== {} ({}) ===", repo.name, repo.path.display());
                        match result {
                            Ok(output) => println!("{}", output),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                }
            }
            RepoCommands::Search { pattern } => {
                let results = repo_mgr.search_files(&pattern).await;
                if results.is_empty() {
                    println!("No matches found for pattern '{}'", pattern);
                } else {
                    println!("Matches for '{}':", pattern);
                    for (repo, matches) in &results {
                        println!("\n  {}:", repo.name);
                        for m in matches {
                            println!("    {}", m);
                        }
                    }
                }
            }
            RepoCommands::Refresh => {
                repo_mgr.refresh_all().await;
                let repos = repo_mgr.list().await;
                println!("Refreshed {} repositories:", repos.len());
                for repo in &repos {
                    println!("  {}", repo.summary());
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

    run_tui(llm_client, skill_manager).await?;

    Ok(())
}

/// Parse agent spec (format: "provider:model" or just "provider")
fn parse_agent_spec(
    spec: &str,
) -> Result<(config::ProviderType, String), Box<dyn std::error::Error + Send + Sync>> {
    if spec.contains(':') {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        let provider = parts[0].parse::<config::ProviderType>()?;
        Ok((provider, parts[1].to_string()))
    } else {
        let provider = spec.parse::<config::ProviderType>()?;
        let model = match provider {
            config::ProviderType::Ollama => "deepseek-r1".to_string(),
            config::ProviderType::OpenRouter => "openrouter/free".to_string(),
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
