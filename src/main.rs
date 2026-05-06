#![deny(warnings)]
mod app;
mod config;
mod desktop;
mod events;
mod llm;
mod tools;
mod ui;
mod git;
mod context;
mod rules;
mod mcp;
mod lsp;
mod jsonrpc;
mod skills;
mod acp;
mod permissions;
mod custom_tools;
mod sessions;
mod web;
mod formatters;
mod audit;
mod stats;
mod planner;
mod rag;
mod telemetry;
mod security;
mod tool_executor;
mod clipboard;

use desktop::detection::get_cinnamon_info;

use app::{App, Mode};
use clipboard::ClipboardManager;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

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
}

#[derive(Subcommand, Debug)]
enum McpCommands {
    /// List available MCP servers
    List,
    /// Install an MCP server by name
    Install { server: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let mut config = config::Config::load();

    // Detect Cinnamon desktop environment and apply theming if available
    let cinnamon_info = get_cinnamon_info();
    if cinnamon_info.desktop.is_cinnamon() {
        // Log desktop detection (silent, for debugging)
        eprintln!("[Desktop] Detected: {} {}", cinnamon_info.desktop, 
            cinnamon_info.version.as_deref().unwrap_or(""));
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
                eprintln!("Usage: opencrust --agent ollama:llama3 --agent gemini:gemini-pro --multi-prompt \"Your question\"");
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

    let _session_manager = Arc::new(Mutex::new(sessions::SessionManager::new()));
    let _session_id = format!("session_{}", chrono::Utc::now().timestamp());

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
                println!("  github    - GitHub integration (npx -y @modelcontextprotocol/server-github)");
                println!("  slack      - Slack integration (npx -y @modelcontextprotocol/server-slack)");
                println!("  filesystem - File system access (npx -y @modelcontextprotocol/server-filesystem)");
                println!("  postgres   - PostgreSQL database (npx -y @modelcontextprotocol/server-postgres)");
                println!("  google-drive - Google Drive (npx -y @modelcontextprotocol/server-google-drive)");
                println!("\nUse `opencrust mcp install <name>` to add a server.");
                println!("For more servers, visit: https://github.com/modelcontextprotocol/servers");
            }
            McpCommands::Install { server } => {
                let config = config::Config::load();
                let mut new_config = config.clone();
                let (command, description) = match server.as_str() {
                    "github" => (vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-github".to_string()], "GitHub integration"),
                    "slack" => (vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-slack".to_string()], "Slack integration"),
                    "filesystem" => (vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()], "File system access"),
                    "postgres" => (vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-postgres".to_string()], "PostgreSQL database"),
                    "google-drive" => (vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-google-drive".to_string()], "Google Drive"),
                    _ => {
                        eprintln!("Unknown MCP server: {}. Use `opencrust mcp list` to see available servers.", server);
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
                println!("Installed MCP server '{}' ({}). Restart open_crust to use it.", server, description);
            }
        }
        return Ok(());
    }

    let (prompt_tx, mut prompt_rx) = mpsc::channel::<String>(32);
    let (response_tx, mut response_rx) = mpsc::channel::<String>(32);
    let (approval_tx, mut approval_rx) = mpsc::channel::<bool>(1);

    let mut client_clone = llm_client.clone();
    tokio::spawn(async move {
        let mut messages_history: Vec<Value> = Vec::new();
        while let Some(prompt) = prompt_rx.recv().await {
            let prompt_str = prompt.trim();
            
            if prompt_str == "/init" {
                match rules::init_project_rules() {
                    Ok(msg) => { let _ = response_tx.send(format!("open_crust: {}", msg)).await; }
                    Err(e) => { let _ = response_tx.send(format!("Error: {}", e)).await; }
                }
                continue;
            } else if prompt_str.starts_with("/provider ") {
                let new_provider = prompt_str.trim_start_matches("/provider ").trim();
                match new_provider.to_lowercase().as_str() {
                    "ollama" => {
                        client_clone.config.provider = config::ProviderType::Ollama;
                        let _ = client_clone.config.save();
                        let _ = response_tx.send(format!("open_crust: Provider switched to Ollama")).await;
                    }
                    "openrouter" => {
                        client_clone.config.provider = config::ProviderType::OpenRouter;
                        let _ = client_clone.config.save();
                        let _ = response_tx.send(format!("open_crust: Provider switched to OpenRouter")).await;
                    }
                    _ => {
                        let _ = response_tx.send(format!("open_crust: Unknown provider '{}'", new_provider)).await;
                    }
                }
                continue;
            } else if prompt_str.starts_with("/model ") {
                let new_model = prompt_str.trim_start_matches("/model ").trim();
                client_clone.config.model = new_model.to_string();
                let _ = client_clone.config.save();
                let _ = response_tx.send(format!("open_crust: Model switched to '{}'", new_model)).await;
                continue;
            } else if prompt_str == "/undo" {
                match git::undo() {
                    Ok(msg) => { let _ = response_tx.send(format!("open_crust: {}", msg)).await; }
                    Err(e) => { let _ = response_tx.send(format!("Error: {}", e)).await; }
                }
                continue;
            } else if prompt_str == "/redo" {
                match git::redo() {
                    Ok(msg) => { let _ = response_tx.send(format!("open_crust: {}", msg)).await; }
                    Err(e) => { let _ = response_tx.send(format!("Error: {}", e)).await; }
                }
                continue;
            }

            let _ = git::checkpoint();
            let enriched_prompt = context::inject_file_context(&prompt);
            let _ = response_tx.send(String::from("open_crust: Thinking...")).await;

            let res = client_clone.send_message(&mut messages_history, &enriched_prompt, response_tx.clone(), &mut approval_rx).await;
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

    let mut app = App::new(llm_client.config.clone(), prompt_tx, approval_tx, llm_client.clone());
    app.refresh_sidebar();

    // Initialize clipboard manager
    let mut clipboard = ClipboardManager::new();

    // Get copy/paste keybinds from config
    let keybinds = app.config.tui.as_ref().map(|t| &t.keybinds);
    let copy_key = keybinds.map(|k| k.copy.clone()).unwrap_or_else(|| "ctrl+c".to_string());
    let paste_key = keybinds.map(|k| k.paste.clone()).unwrap_or_else(|| "ctrl+v".to_string());
    let exit_keys = keybinds.map(|k| k.app_exit.clone()).unwrap_or_else(|| "ctrl+q".to_string());
    let submit_keys = keybinds.map(|k| k.input_submit.clone()).unwrap_or_else(|| "return".to_string());

    loop {
        while let Ok(response) = response_rx.try_recv() {
            if response.contains("[APPROVAL_REQUIRED]") {
                app.waiting_for_approval = true;
            } else if response.starts_with("[DIFF_REQUIRED]") {
                let parts: Vec<&str> = response.strip_prefix("[DIFF_REQUIRED]").unwrap().splitn(3, '|').collect();
                if parts.len() == 3 {
                    app.proposed_changes.push(crate::app::ProposedChange {
                        original: parts[1].to_string(),
                        proposed: parts[2].to_string(),
                    });
                    app.mode = crate::app::Mode::Review;
                }
            }
            
            let tab = &mut app.tabs[0]; // always push to Chat tab
            if let Some(last) = tab.messages.last() {
                if last == "open_crust: Thinking..." || last.starts_with("open_crust: Executing tool") {
                    tab.messages.pop();
                }
            }
            tab.messages.push(response);
        }

        terminal.draw(|f| ui::draw(f, &app))?;

        if let Some(Event::Key(key)) = events::next_event().await? {
            // Check for Copy (Ctrl+C) - copy current input to clipboard
            if check_key_match(&key, &copy_key) {
                if !app.input.is_empty() {
                    if clipboard.copy(&app.input) {
                        app.tabs[0].messages.push(String::from("Copied to clipboard"));
                    }
                }
                continue;
            }

            // Check for Paste (Ctrl+V) - paste from clipboard to input
            if check_key_match(&key, &paste_key) {
                if let Some(text) = clipboard.paste() {
                    app.input.push_str(&text);
                }
                continue;
            }

            if app.waiting_for_approval {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
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
                if key.modifiers == crossterm::event::KeyModifiers::CONTROL && key.code == KeyCode::Char('b') {
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
                                KeyCode::Esc => {
                                    app.enter_normal_mode();
                                }
                                KeyCode::Backspace => {
                                    app.handle_backspace();
                                }
                                KeyCode::Char(c) => {
                                    app.handle_char(c);
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
                    },
                    Mode::Review => match key.code {
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            app.mode = Mode::Normal;
                            if let Some(tx) = &app.approval_tx {
                                let _ = tx.try_send(true);
                            }
                            app.tabs[0].messages.push(String::from("You: a (Approved Change)"));
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            app.mode = Mode::Normal;
                            if let Some(tx) = &app.approval_tx {
                                let _ = tx.try_send(false);
                            }
                            app.tabs[0].messages.push(String::from("You: d (Denied Change)"));
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
                            if let Some((name, _, cmd)) = app.mcp_browser_items.get(app.mcp_browser_selected) {
                                if !app.config.mcp.contains_key(name) {
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
                        }
                        KeyCode::Char(c) => {
                            app.mcp_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.mcp_input.pop();
                        }
                        _ => {}
                    },
                }
            }
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

        if let Some(code) = target_code {
            if key.code == code && key.modifiers.contains(target_modifiers) {
                return true;
            }
        }
    }
    false
}

/// Parse agent spec (format: "provider:model" or just "provider")
fn parse_agent_spec(spec: &str) -> Result<(config::ProviderType, String), Box<dyn std::error::Error + Send + Sync>> {
    if spec.contains(':') {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        let provider = match parts[0].to_lowercase().as_str() {
            "ollama" => config::ProviderType::Ollama,
            "openrouter" => config::ProviderType::OpenRouter,
            "openai" => config::ProviderType::OpenAI,
            "gemini" => config::ProviderType::Gemini,
            _ => return Err(format!("Unknown provider: {}", parts[0]).into()),
        };
        Ok((provider, parts[1].to_string()))
    } else {
        let provider = match spec.to_lowercase().as_str() {
            "ollama" => config::ProviderType::Ollama,
            "openrouter" => config::ProviderType::OpenRouter,
            "openai" => config::ProviderType::OpenAI,
            "gemini" => config::ProviderType::Gemini,
            _ => return Err(format!("Unknown provider: {}", spec).into()),
        };
        let model = match provider {
            config::ProviderType::Ollama => "llama3".to_string(),
            config::ProviderType::OpenRouter => "openai/gpt-4".to_string(),
            config::ProviderType::OpenAI => "gpt-4".to_string(),
            config::ProviderType::Gemini => "gemini-pro".to_string(),
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
    mcp_manager.lock().await.load_from_config(&base_config.mcp).await;

    let lsp_manager = Arc::new(Mutex::new(lsp::LspManager::new()));
    lsp_manager.lock().await.load_from_config(&base_config.lsp).await;

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
            let llm_client = llm::LlmClient::new(
                config,
                mcp_mgr,
                lsp_mgr,
                skill_mgr,
                custom_mgr,
            );

            let provider_name = format!("{:?}", provider);
            print!("Agent {} ({}) thinking...\n", provider_name, model);

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
