//! Normal mode handler

use crate::app::{App, Message, Mode, PlanMode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct NormalHandler;

#[async_trait::async_trait]
impl ModeHandler for NormalHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        _ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        match key.code {
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
                ModeAction::Continue
            }
            KeyCode::Down => {
                // Scroll message list down
                app.message_scroll = app.message_scroll.saturating_sub(1);
                ModeAction::Continue
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
                ModeAction::Continue
            }
            KeyCode::PageDown => {
                // Scroll down by 10 lines
                app.message_scroll = app.message_scroll.saturating_sub(10);
                ModeAction::Continue
            }
            KeyCode::Home => {
                // Scroll to the very top (clamped in draw_message_list)
                app.message_scroll = usize::MAX;
                ModeAction::Continue
            }
            KeyCode::End => {
                app.message_scroll = 0;
                ModeAction::Continue
            }
            KeyCode::Char('?') => {
                app.mode = Mode::Help;
                ModeAction::SwitchMode(Mode::Help)
            }
            // Sidebar navigation: [ = up, ] = down
            KeyCode::Char('[')
                if app.show_sidebar
                    && app.sidebar_selected > 0
                    && !app.sidebar_items.is_empty() =>
            {
                app.sidebar_selected -= 1;
                ModeAction::Continue
            }
            KeyCode::Char(']')
                if app.show_sidebar
                    && app.sidebar_selected + 1 < app.sidebar_items.len()
                    && !app.sidebar_items.is_empty() =>
            {
                app.sidebar_selected += 1;
                ModeAction::Continue
            }
            KeyCode::Char('i') => {
                app.enter_insert_mode();
                ModeAction::SwitchMode(Mode::Insert)
            }
            KeyCode::Char('s') => {
                app.mode = Mode::Servers;
                ModeAction::SwitchMode(Mode::Servers)
            }
            KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                // Toggle plan mode
                match app.plan_mode {
                    PlanMode::Disabled => {
                        app.plan_mode = PlanMode::Planning;
                        app.llm_client
                            .set_plan_mode(crate::llm::PlanModeState::Planning);
                        app.tabs[0].messages.push(Message::new(String::from(
                            "Entering plan mode — write tools blocked. Press Ctrl+P to exit.",
                        )));
                    }
                    PlanMode::Planning => {
                        app.plan_mode = PlanMode::Disabled;
                        app.llm_client
                            .set_plan_mode(crate::llm::PlanModeState::Disabled);
                        app.tabs[0].messages.push(Message::new(String::from(
                            "Exiting plan mode — write tools enabled.",
                        )));
                    }
                }
                ModeAction::Continue
            }
            KeyCode::Char('k') if key.modifiers == KeyModifiers::CONTROL => {
                app.mode = Mode::CommandPalette;
                ModeAction::SwitchMode(Mode::CommandPalette)
            }
            KeyCode::Char('K')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                app.mode = Mode::SkillBrowser;
                ModeAction::SwitchMode(Mode::SkillBrowser)
            }
            KeyCode::Char('P')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                app.mode = Mode::PluginBrowser;
                ModeAction::SwitchMode(Mode::PluginBrowser)
            }
            KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                // Spawn background task with current input
                if !app.input.is_empty() {
                    let prompt = app.input.clone();
                    crate::event_loop::background_tasks::spawn_background_task(app, prompt);
                    app.input.clear();
                    app.tabs[0]
                        .messages
                        .push(Message::new(String::from("Spawning background task...")));
                } else {
                    app.tabs[0].messages.push(Message::new(String::from(
                        "No input to spawn as background task",
                    )));
                }
                ModeAction::Continue
            }
            KeyCode::Tab => {
                app.active_tab = (app.active_tab + 1) % app.tabs.len();
                ModeAction::Continue
            }
            KeyCode::Char('v') if key.modifiers == KeyModifiers::ALT => {
                app.vim_mode = !app.vim_mode;
                let mode_str = if app.vim_mode { "enabled" } else { "disabled" };
                app.tabs[0]
                    .messages
                    .push(Message::new(format!("Vim Mode {}", mode_str)));
                ModeAction::Continue
            }
            KeyCode::Char('m') if key.modifiers == KeyModifiers::CONTROL => {
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
                app.mcp_showcase_ui = Some(crate::mcp_showcase::McpShowcaseUI::new(servers));
                ModeAction::SwitchMode(Mode::McpShowcase)
            }
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
                app.mode = Mode::MissionControl;
                if app.mission_control_ui.is_none() {
                    app.mission_control_ui = Some(crate::mission_control::MissionControlUI::new());
                }
                // Refresh tasks from orchestrator bridge
                if let Some(ref tasks_arc) = app.orchestrator_tasks
                    && let Some(ref mut ui) = app.mission_control_ui
                {
                    ui.refresh_tasks(Some(tasks_arc));
                }
                ModeAction::SwitchMode(Mode::MissionControl)
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                // Format current file from sidebar selection
                let msg = app.format_current_file();
                app.tabs[0].messages.push(Message::new(msg));
                ModeAction::Continue
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                // Toggle split view mode for diffs
                app.mode = Mode::SplitView;
                ModeAction::SwitchMode(Mode::SplitView)
            }
            _ => ModeAction::Continue,
        }
    }
}
