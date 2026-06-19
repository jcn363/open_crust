//! Mode handlers for the TUI event loop
//!
//! Each mode has its own handler module implementing the `ModeHandler` trait.
//! The `dispatch_mode` function routes key events to the appropriate handler.

pub mod command_palette;
pub mod help;
pub mod insert;
pub mod mcp_showcase;
pub mod mission_control;
pub mod normal;
pub mod plugin_browser;
pub mod review;
pub mod servers;
pub mod skill_browser;
pub mod split_view;
pub mod types;

pub use types::{HandlerContext, ModeAction, ModeHandler};

use crate::app::{App, Mode};
use crossterm::event::KeyEvent;

/// Dispatch a key event to the appropriate mode handler
pub async fn dispatch_mode(
    app: &mut App,
    key: KeyEvent,
    ctx: &mut HandlerContext<'_>,
) -> ModeAction {
    // Create the appropriate handler for the current mode
    let mut handler: Box<dyn ModeHandler + Send> = match app.mode {
        Mode::Normal => Box::new(normal::NormalHandler),
        Mode::Insert => Box::new(insert::InsertHandler),
        Mode::Review => Box::new(review::ReviewHandler),
        Mode::Servers => Box::new(servers::ServersHandler),
        Mode::SkillBrowser => Box::new(skill_browser::SkillBrowserHandler),
        Mode::PluginBrowser => Box::new(plugin_browser::PluginBrowserHandler),
        Mode::CommandPalette => Box::new(command_palette::CommandPaletteHandler),
        Mode::Help => Box::new(help::HelpHandler),
        Mode::McpShowcase => Box::new(mcp_showcase::McpShowcaseHandler),
        Mode::MissionControl => Box::new(mission_control::MissionControlHandler),
        Mode::SplitView => Box::new(split_view::SplitViewHandler),
    };

    handler.handle_key(app, key, ctx).await
}
