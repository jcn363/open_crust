//! MCP Showcase mode handler

use crate::app::{App, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::KeyEvent;

pub struct McpShowcaseHandler;

#[async_trait::async_trait]
impl ModeHandler for McpShowcaseHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        _ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        if let Some(ref mut ui) = app.mcp_showcase_ui {
            match ui.handle_key(key.code) {
                crate::mcp_showcase::McpShowcaseAction::ToggleServer(name) => {
                    // Toggle enabled status in config
                    if let Some(server_cfg) = app.config.mcp.get_mut(&name) {
                        server_cfg.enabled = !server_cfg.enabled;
                        // Save updated config to disk
                        app.config.save();
                        // Update UI server list to reflect change
                        ui.toggle_server(&name);
                    }
                    ModeAction::Continue
                }
                crate::mcp_showcase::McpShowcaseAction::ExitMode => {
                    app.mode = Mode::Normal;
                    ModeAction::SwitchMode(Mode::Normal)
                }
                crate::mcp_showcase::McpShowcaseAction::None => ModeAction::Continue,
            }
        } else {
            ModeAction::Continue
        }
    }
}
