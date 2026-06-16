//! Servers mode handler

use crate::app::{App, Message, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent};

pub struct ServersHandler;

#[async_trait::async_trait]
impl ModeHandler for ServersHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        _ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        match key.code {
            KeyCode::Esc => {
                if !app.mcp_input.is_empty() {
                    app.mcp_input.clear();
                } else {
                    app.mode = Mode::Normal;
                    return ModeAction::SwitchMode(Mode::Normal);
                }
                ModeAction::Continue
            }
            KeyCode::Up => {
                if app.mcp_browser_selected > 0 {
                    app.mcp_browser_selected -= 1;
                }
                // Adjust scroll if needed
                if app.mcp_browser_selected < app.mcp_browser_scroll {
                    app.mcp_browser_scroll = app.mcp_browser_selected;
                }
                ModeAction::Continue
            }
            KeyCode::Down => {
                if app.mcp_browser_selected < app.mcp_browser_items.len() - 1 {
                    app.mcp_browser_selected += 1;
                }
                // Adjust scroll if needed (assuming 20 visible items)
                if app.mcp_browser_selected >= app.mcp_browser_scroll + 20 {
                    app.mcp_browser_scroll = app.mcp_browser_selected - 19;
                }
                ModeAction::Continue
            }
            KeyCode::Enter => {
                // Install the selected server
                if let Some((name, _, cmd)) = app.mcp_browser_items.get(app.mcp_browser_selected)
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
                ModeAction::Continue
            }
            KeyCode::Char(c) => {
                app.mcp_input.push(c);
                ModeAction::Continue
            }
            KeyCode::Backspace => {
                app.mcp_input.pop();
                ModeAction::Continue
            }
            _ => ModeAction::Continue,
        }
    }
}
