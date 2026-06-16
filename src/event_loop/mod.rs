//! Interactive TUI event loop and key handling
//!
//! Contains the main rendering loop, keyboard dispatch, input prediction,
//! clipboard integration, background task handling, and skill hot-reload.
//! Extracted from the main entry point to keep startup logic focused.

use crate::{
    app::{App, Message},
    clipboard::ClipboardManager,
    event_loop::modes::{HandlerContext, dispatch_mode},
    llm, ui,
};
use crossterm::{
    ExecutableCommand,
    event::{Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

mod background_tasks;
mod frame_limiter;
mod input_prediction;
mod keybinds;
mod modes;
mod response_handler;
mod share;
mod skill_hot_reload;
mod slash_commands;

/// Run the interactive TUI event loop.
///
/// Sets up channels, spawns the LLM background task, initializes the terminal
/// and application state, then enters the main render/input loop until the
/// user requests an exit.
pub async fn run_tui(
    llm_client: llm::LlmClient,
    skill_manager: Arc<Mutex<crate::skills::SkillManager>>,
    plugin_manager: Arc<Mutex<crate::plugins::PluginManager>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

            // Handle slash commands and LLM queries
            slash_commands::handle_slash_command(
                prompt_str,
                &client_clone,
                &response_tx,
                &mut messages_history,
                &mut approval_rx,
            )
            .await;
        }
    });

    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    let mut app = App::new(
        (*llm_client.config).clone(),
        prompt_tx,
        approval_tx,
        background_task_tx,
        llm_client.clone(),
    );
    // Wire Mission Control bridge to orchestrator shared task state
    app.orchestrator_tasks = Some(llm_client.orchestrator_tasks.clone());
    app.refresh_sidebar();

    // Frame rate limiter for smoother UI
    let mut frame_limiter = frame_limiter::FrameLimiter::new();

    // Populate skill browser items from skill manager
    {
        let skills = skill_manager.lock().await;
        let skill_list = skills.list_skills_with_stats();
        for (name, description, active) in skill_list {
            app.skill_browser_items.push((name, description, active));
        }
    }

    // Populate plugin browser items from plugin manager
    {
        let plugins = plugin_manager.lock().await;
        for plugin in plugins.list() {
            app.plugin_browser_items.push((
                plugin.name.clone(),
                plugin.description.clone(),
                plugin.enabled,
            ));
        }
    }

    // Initialize clipboard manager
    let clipboard = Arc::new(Mutex::new(ClipboardManager::new()));

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
    let _submit_keys = keybinds
        .map(|k| k.input_submit.clone())
        .unwrap_or_else(|| "return".to_string());

    // Create handler context with all dependencies for mode handlers
    let mut handler_ctx = HandlerContext::new(
        skill_manager.clone(),
        plugin_manager.clone(),
        clipboard.clone(),
        &llm_client,
        app.orchestrator_tasks.clone(),
    );

    loop {
        // Handle incoming responses from LLM task
        response_handler::handle_responses(&mut app, &mut response_rx);

        // Handle background task notifications
        background_tasks::handle_background_tasks(&mut app, &mut background_task_rx);

        // Periodic skill hot-reload check
        skill_hot_reload::check_skill_updates(&mut app, &skill_manager).await;

        // Frame rate limiting for smoother UI
        frame_limiter.wait_for_next_frame().await;
        // Only redraw when state has changed (dirty flag)
        if app.dirty {
            terminal.draw(|f| ui::draw(f, &mut app))?;
            app.dirty = false;
        }

        if let Some(Event::Key(key)) = crate::events::next_event()? {
            app.mark_dirty();

            // Check for Copy (Ctrl+C) - copy current input to clipboard
            if keybinds::check_key_match(&key, &copy_key) {
                if !app.input.is_empty() && clipboard.lock().await.copy(&app.input) {
                    if let Some(tab) = app.tabs.get_mut(app.active_tab) {
                        tab.messages
                            .push(Message::new(String::from("Copied to clipboard")));
                    }
                }
                continue;
            }

            // Check for Paste (Ctrl+V) - paste from clipboard to input
            if keybinds::check_key_match(&key, &paste_key) {
                if let Some(text) = clipboard.lock().await.paste() {
                    app.input.push_str(&text);
                    // Update last input time for prediction
                    app.last_input_time = Some(std::time::Instant::now());
                }
                continue;
            }

            // Handle approval waiting
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
                continue;
            }

            // Check for exit keys
            if keybinds::check_key_match(&key, &exit_keys) {
                app.should_quit = true;
            }

            // Global keys
            if key.modifiers == crossterm::event::KeyModifiers::CONTROL
                && key.code == KeyCode::Char('b')
            {
                app.show_sidebar = !app.show_sidebar;
                continue;
            }

            // Dispatch to mode handler
            let action = dispatch_mode(&mut app, key, &mut handler_ctx).await;
            match action {
                crate::event_loop::modes::ModeAction::Continue => {}
                crate::event_loop::modes::ModeAction::ExitMode => {
                    app.mode = crate::app::Mode::Normal;
                }
                crate::event_loop::modes::ModeAction::SwitchMode(mode) => {
                    app.mode = mode;
                }
                crate::event_loop::modes::ModeAction::Quit => {
                    app.should_quit = true;
                }
            }
        }
        // Check for prediction results
        input_prediction::handle_prediction_results(&mut app, &mut prediction_rx);

        // Trigger prediction if needed (after 300ms debounce)
        let llm_client_clone = app.llm_client.clone();
        input_prediction::trigger_prediction_if_needed(&mut app, &llm_client_clone, &prediction_tx);

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
