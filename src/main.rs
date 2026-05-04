#![deny(warnings)]
mod app;
mod config;
mod events;
mod llm;
mod tools;
mod ui;
mod git;
mod context;
mod rules;
mod mcp;
mod lsp;
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

use app::{App, Mode};
use chrono;
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
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start in ACP mode (JSON-RPC over stdio)
    Acp,
    /// Run a single command and exit
    Run { command: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let config = config::Config::load();

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
            
            if let Some(last) = app.messages.last() {
                if last == "open_crust: Thinking..." || last.starts_with("open_crust: Executing tool") {
                    app.messages.pop();
                }
            }
            app.messages.push(response);
        }

        terminal.draw(|f| ui::draw(f, &app))?;

        if let Some(Event::Key(key)) = events::next_event().await? {
            let keybinds = app.config.tui.as_ref().map(|t| &t.keybinds);
            let exit_keys = keybinds.map(|k| k.app_exit.as_str()).unwrap_or("ctrl+c,ctrl+d");
            let submit_keys = keybinds.map(|k| k.input_submit.as_str()).unwrap_or("return");

            if app.waiting_for_approval {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.waiting_for_approval = false;
                        if let Some(tx) = &app.approval_tx {
                            let _ = tx.try_send(true);
                        }
                        app.messages.push(String::from("You: y (Approved)"));
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.waiting_for_approval = false;
                        if let Some(tx) = &app.approval_tx {
                            let _ = tx.try_send(false);
                        }
                        app.messages.push(String::from("You: n (Denied)"));
                    }
                    _ => {}
                }
            } else {
                // Check for exit keys
                if check_key_match(&key, exit_keys) {
                    app.should_quit = true;
                }

                match app.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('i') => {
                            app.enter_insert_mode();
                        }
                        KeyCode::Char('s') => {
                            app.mode = Mode::Servers;
                        }
                        _ => {}
                    },
                    Mode::Insert => {
                        if check_key_match(&key, submit_keys) {
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
                            app.messages.push(String::from("You: a (Approved Change)"));
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            app.mode = Mode::Normal;
                            if let Some(tx) = &app.approval_tx {
                                let _ = tx.try_send(false);
                            }
                            app.messages.push(String::from("You: d (Denied Change)"));
                        }
                        _ => {}
                    },
                    Mode::Servers => match key.code {
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.mcp_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.mcp_input.pop();
                        }
                        KeyCode::Enter => {
                            if !app.mcp_input.is_empty() {
                                let parts: Vec<&str> = app.mcp_input.splitn(2, '=').collect();
                                if parts.len() == 2 {
                                    let name = parts[0].trim();
                                    let command = parts[1].trim();
                                    app.config.mcp.insert(name.to_string(), crate::config::McpConfig {
                                        command: vec![command.to_string()],
                                        environment: Some(std::collections::HashMap::new()),
                                        enabled: true,
                                    });
                                    app.config.save();
                                    app.messages.push(format!("System: Added MCP server '{}'", name));
                                    app.mcp_input.clear();
                                }
                            }
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
