//! Interactive TUI event loop and key handling
//!
//! Contains the main rendering loop, keyboard dispatch, input prediction,
//! clipboard integration, background task handling, and skill hot-reload.
//! Extracted from the main entry point to keep startup logic focused.

use crate::{
    app::{App, Message, Mode},
    clipboard::ClipboardManager,
    context, git, llm, mcp_showcase, mission_control, rules, ui,
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
                        new_config.provider = crate::config::ProviderType::Ollama;
                        new_config.save();
                        let _ = response_tx
                            .send("opencrust: Provider switched to Ollama".to_string())
                            .await;
                    }
                    "openrouter" => {
                        new_config.provider = crate::config::ProviderType::OpenRouter;
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
            } else if prompt_str.starts_with("/goal ") {
                let goal_desc = prompt_str.trim_start_matches("/goal ").trim();
                if goal_desc.is_empty() {
                    let _ = response_tx
                        .send("opencrust: Usage: /goal <description>".to_string())
                        .await;
                } else {
                    client_clone.set_goal(goal_desc.to_string());
                    let _ = response_tx
                        .send(format!(
                            "opencrust: Goal set: '{}'. Agent will work autonomously until completed. Use /goal-clear to reset.",
                            goal_desc
                        ))
                        .await;
                }
                continue;
            } else if prompt_str == "/goal-clear" || prompt_str == "/goal clear" {
                client_clone.clear_goal();
                let _ = response_tx
                    .send("opencrust: Goal cleared.".to_string())
                    .await;
                continue;
            } else if prompt_str == "/goal-status" || prompt_str == "/goal status" {
                match client_clone.get_goal() {
                    Some(goal) => {
                        let _ = response_tx
                            .send(format!(
                                "opencrust: Active goal: '{}' (set {})",
                                goal.description,
                                goal.created_at.format("%Y-%m-%d %H:%M")
                            ))
                            .await;
                    }
                    None => {
                        let _ = response_tx
                            .send("opencrust: No active goal.".to_string())
                            .await;
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
                    app.tabs[0].messages.push(Message::new(format!(
                        "System: Discovered new skills: {}",
                        added.join(", ")
                    )));
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
                    app.tabs[0].messages.push(Message::new(format!(
                        "System: Removed skills: {}",
                        removed.join(", ")
                    )));
                    app.skill_browser_items
                        .retain(|(name, _, _)| !removed.contains(name));
                }
                if !modified.is_empty() {
                    app.tabs[0].messages.push(Message::new(format!(
                        "System: Modified skills: {}",
                        modified.join(", ")
                    )));
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

        if let Some(Event::Key(key)) = crate::events::next_event()? {
            // Check for Copy (Ctrl+C) - copy current input to clipboard
            if check_key_match(&key, &copy_key) {
                if !app.input.is_empty() && clipboard.copy(&app.input) {
                    if let Some(tab) = app.tabs.get_mut(app.active_tab) {
                        tab.messages
                            .push(Message::new(String::from("Copied to clipboard")));
                    }
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
                            // Toggle plan mode
                            match app.plan_mode {
                                crate::app::PlanMode::Disabled => {
                                    app.plan_mode = crate::app::PlanMode::Planning;
                                    app.llm_client
                                        .set_plan_mode(crate::llm::PlanModeState::Planning);
                                    app.tabs[0]
                                        .messages
                                        .push(Message::new(String::from("Entering plan mode — write tools blocked. Press Ctrl+P to exit.")));
                                }
                                crate::app::PlanMode::Planning => {
                                    app.plan_mode = crate::app::PlanMode::Disabled;
                                    app.llm_client
                                        .set_plan_mode(crate::llm::PlanModeState::Disabled);
                                    app.tabs[0].messages.push(Message::new(String::from(
                                        "Exiting plan mode — write tools enabled.",
                                    )));
                                }
                            }
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
                        KeyCode::Char('P')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL)
                                && key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::SHIFT) =>
                        {
                            app.mode = Mode::PluginBrowser;
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
                        KeyCode::Char('f')
                            if key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                        {
                            // Format current file from sidebar selection
                            let msg = app.format_current_file();
                            app.tabs[0].messages.push(Message::new(msg));
                        }
                        _ => {}
                    },
                    Mode::Insert => {
                        if check_key_match(&key, &submit_keys) {
                            // Handle slash commands before submitting
                            let input_trimmed = app.input.trim();
                            if input_trimmed == "/share" {
                                let share_path = share_conversation(&app);
                                if let Some(path) = share_path {
                                    let _ = clipboard.copy(&path);
                                    app.tabs[0].messages.push(Message::new(format!(
                                        "Conversation shared to: {}\nPath copied to clipboard.",
                                        path
                                    )));
                                } else {
                                    app.tabs[0].messages.push(Message::new(
                                        "Error: Failed to share conversation.".to_string(),
                                    ));
                                }
                                app.input.clear();
                            } else if input_trimmed == "/format" {
                                let msg = app.format_current_file();
                                app.tabs[0].messages.push(Message::new(msg));
                                app.input.clear();
                            } else if input_trimmed.starts_with("/format ") {
                                let path = input_trimmed.trim_start_matches("/format ").trim();
                                match crate::formatters::format_file(std::path::Path::new(path)) {
                                    Ok(_) => {
                                        app.tabs[0]
                                            .messages
                                            .push(Message::new(format!("Formatted {}", path)));
                                    }
                                    Err(e) => {
                                        app.tabs[0].messages.push(Message::new(e));
                                    }
                                }
                                app.input.clear();
                            } else if input_trimmed.starts_with('/') {
                                // Check for custom commands
                                let cmd_name = input_trimmed
                                    .trim_start_matches('/')
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("");
                                let args = input_trimmed
                                    .trim_start_matches('/')
                                    .strip_prefix(cmd_name)
                                    .unwrap_or("")
                                    .trim();
                                if app.custom_commands.has_command(cmd_name) {
                                    match app.custom_commands.execute_command(cmd_name, args) {
                                        Ok(output) => {
                                            app.tabs[0].messages.push(Message::new(format!(
                                                "/{}: {}",
                                                cmd_name, output
                                            )));
                                        }
                                        Err(e) => {
                                            app.tabs[0].messages.push(Message::new(format!(
                                                "/{} error: {}",
                                                cmd_name, e
                                            )));
                                        }
                                    }
                                    app.input.clear();
                                } else {
                                    app.submit_message();
                                }
                            } else {
                                app.submit_message();
                            }
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
                            // File picker is active — handle navigation
                            if app.file_picker_active {
                                match key.code {
                                    KeyCode::Esc => {
                                        app.cancel_file_picker();
                                    }
                                    KeyCode::Up if app.file_picker_selected > 0 => {
                                        app.file_picker_selected -= 1;
                                        // Auto-scroll
                                        if app.file_picker_selected < app.file_picker_scroll {
                                            app.file_picker_scroll = app.file_picker_selected;
                                        }
                                    }
                                    KeyCode::Down
                                        if app.file_picker_selected + 1
                                            < app.file_picker_results.len() =>
                                    {
                                        app.file_picker_selected += 1;
                                        // Auto-scroll
                                        let max_visible = 9; // matches picker height - 3
                                        if app.file_picker_selected
                                            >= app.file_picker_scroll + max_visible
                                        {
                                            app.file_picker_scroll =
                                                app.file_picker_selected - max_visible + 1;
                                        }
                                    }
                                    KeyCode::Enter => {
                                        if let Some(path) = app.confirm_file_picker() {
                                            // Insert file reference into input
                                            app.input.push_str(&format!("@\"{}\" ", path));
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        if app.file_picker_query.is_empty() {
                                            app.cancel_file_picker();
                                        } else {
                                            app.file_picker_query.pop();
                                            app.file_picker_selected = 0;
                                            app.file_picker_scroll = 0;
                                            app.file_picker_results =
                                                app.filter_project_files(&app.file_picker_query);
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        app.file_picker_query.push(c);
                                        app.file_picker_selected = 0;
                                        app.file_picker_scroll = 0;
                                        app.file_picker_results =
                                            app.filter_project_files(&app.file_picker_query);
                                    }
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Char('@') => {
                                        // Activate file picker with current input as query
                                        let query = app.input.clone();
                                        app.activate_file_picker(query);
                                    }
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
                    }
                    Mode::Review => match key.code {
                        // Navigation between files
                        KeyCode::Up if app.plan_review_index > 0 => {
                            app.plan_review_index -= 1;
                            app.plan_review_scroll = 0;
                        }
                        KeyCode::Down if app.plan_review_index + 1 < app.proposed_changes.len() => {
                            app.plan_review_index += 1;
                            app.plan_review_scroll = 0;
                        }
                        // Scroll diff view (j/k)
                        KeyCode::Char('j') => {
                            app.plan_review_scroll = app.plan_review_scroll.saturating_add(1);
                        }
                        KeyCode::Char('k') => {
                            app.plan_review_scroll = app.plan_review_scroll.saturating_sub(1);
                        }
                        // Toggle unified / side-by-side view ('u')
                        KeyCode::Char('u') => {
                            app.review_show_unified = !app.review_show_unified;
                            app.plan_review_scroll = 0;
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
                            if !app.mcp_input.is_empty() {
                                app.mcp_input.clear();
                            } else {
                                app.mode = Mode::Normal;
                            }
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
                                let mcp_config = crate::config::McpConfig {
                                    command: cmd.clone(),
                                    environment: None,
                                    enabled: true,
                                };
                                app.config.mcp.insert(name.clone(), mcp_config);
                                app.config.save();
                                app.tabs[0].messages.push(Message::new(format!(
                                    "System: Installed MCP server '{}'. Restart opencrust to use it.",
                                    name
                                )));
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
                    Mode::PluginBrowser => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Up => {
                            if app.plugin_browser_selected > 0 {
                                app.plugin_browser_selected -= 1;
                            }
                            if app.plugin_browser_selected < app.plugin_browser_scroll {
                                app.plugin_browser_scroll = app.plugin_browser_selected;
                            }
                        }
                        KeyCode::Down => {
                            if app.plugin_browser_selected < app.plugin_browser_items.len() - 1 {
                                app.plugin_browser_selected += 1;
                            }
                            if app.plugin_browser_selected >= app.plugin_browser_scroll + 20 {
                                app.plugin_browser_scroll = app.plugin_browser_selected - 19;
                            }
                        }
                        KeyCode::Enter => {
                            if let Some((name, _, enabled)) = app
                                .plugin_browser_items
                                .get_mut(app.plugin_browser_selected)
                            {
                                let new_enabled = !*enabled;
                                *enabled = new_enabled;

                                // Update plugin_manager
                                let plugin_name = name.clone();
                                let pm = plugin_manager.clone();
                                tokio::spawn(async move {
                                    let mut plugins = pm.lock().await;
                                    if new_enabled {
                                        let _ = plugins.enable(&plugin_name);
                                    } else {
                                        let _ = plugins.disable(&plugin_name);
                                    }
                                });

                                let status = if new_enabled { "enabled" } else { "disabled" };
                                app.tabs[0].messages.push(Message::new(format!(
                                    "System: Plugin '{}' {}",
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
                        KeyCode::Down if app.command_palette_selected < 3 => {
                            // 4 items total (0-3)
                            app.command_palette_selected += 1;
                        }
                        KeyCode::Enter => {
                            // Handle command palette selection
                            match app.command_palette_selected {
                                0 => {
                                    // Switch Provider
                                    let providers = [
                                        crate::config::ProviderType::Ollama,
                                        crate::config::ProviderType::OpenRouter,
                                        crate::config::ProviderType::OpenAI,
                                        crate::config::ProviderType::Gemini,
                                        crate::config::ProviderType::Mistral,
                                        crate::config::ProviderType::Anthropic,
                                        crate::config::ProviderType::Groq,
                                        crate::config::ProviderType::TogetherAi,
                                        crate::config::ProviderType::Replicate,
                                        crate::config::ProviderType::DeepSeek,
                                        crate::config::ProviderType::LocalAi,
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
                                    app.tabs[0].messages.push(Message::new(format!(
                                        "Model switching not fully implemented yet. Current model: {}",
                                        app.config.model
                                    )));
                                    app.mode = Mode::Normal;
                                }
                                2 => {
                                    // Clear Context
                                    app.tabs[0].messages.clear();
                                    app.tabs[0].messages.push(Message::new(String::from(
                                        "Welcome to opencrust. Press 'i' to type, 'Tab' to switch tabs, 'Ctrl+Q' to quit.",
                                    )));
                                    app.history.clear();
                                    app.save_history();
                                    app.tabs[0]
                                        .messages
                                        .push(Message::new("Context cleared.".to_string()));
                                    app.mode = Mode::Normal;
                                }
                                3 => {
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
                                mcp_showcase::McpShowcaseAction::ToggleServer(name) => {
                                    // Toggle enabled status in config
                                    if let Some(server_cfg) = app.config.mcp.get_mut(&name) {
                                        server_cfg.enabled = !server_cfg.enabled;
                                        // Save updated config to disk
                                        app.config.save();
                                        // Update UI server list to reflect change
                                        ui.toggle_server(&name);
                                    }
                                }
                                mcp_showcase::McpShowcaseAction::ExitMode => {
                                    app.mode = Mode::Normal;
                                }
                                mcp_showcase::McpShowcaseAction::None => {}
                            }
                        }
                    }
                    Mode::MissionControl => {
                        if let Some(ref mut ui) = app.mission_control_ui {
                            // Refresh tasks from orchestrator bridge before handling input
                            if let Some(ref tasks_arc) = app.orchestrator_tasks {
                                ui.refresh_tasks(Some(tasks_arc));
                            }
                            if let mission_control::MissionControlAction::ExitMode =
                                ui.handle_key(key.code)
                            {
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
    std::io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

/// Serialize the current tab's conversation to a shareable JSON file.
/// Returns the file path on success.
fn share_conversation(app: &App) -> Option<String> {
    use chrono::Utc;
    use serde_json::json;
    use std::fs;

    let tab = app.tabs.get(app.active_tab)?;
    if tab.messages.is_empty() {
        return None;
    }

    let share_data = json!({
        "version": 1,
        "project": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
        "provider": app.config.provider.to_string(),
        "model": app.config.model,
        "tab": tab.name,
        "created_at": Utc::now().to_rfc3339(),
        "messages": tab.messages.iter().map(|m| {
            json!({
                "timestamp": m.timestamp.to_rfc3339(),
                "content": m.content,
            })
        }).collect::<Vec<_>>(),
        "message_count": tab.messages.len(),
    });

    let share_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".config/opencrust/shares");
    if fs::create_dir_all(&share_dir).is_err() {
        return None;
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("share-{}.json", timestamp);
    let share_path = share_dir.join(&filename);

    let content = serde_json::to_string_pretty(&share_data).ok()?;
    fs::write(&share_path, content).ok()?;

    share_path.to_str().map(|s| s.to_string())
}

fn check_key_match(key: &crossterm::event::KeyEvent, keybind_str: &str) -> bool {
    use crossterm::event::KeyModifiers;

    for combo in keybind_str.split(',') {
        let mut target_modifiers = KeyModifiers::empty();
        let mut target_code = None;

        for part in combo.trim().split('+') {
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
