//! Popup rendering for the TUI
//!
//! Draws modal overlays: review confirmation, MCP server browser, command
//! palette, skill browser, and help popup. Each popup is a self-contained
//! rendering function with keyboard navigation.

mod browsers;
mod file_picker;
mod helpers;
mod review;
mod simple;

pub use browsers::{draw_plugin_browser, draw_servers_popup, draw_skill_browser};
pub use file_picker::draw_file_picker;
pub use review::draw_review_popup;
pub use simple::{draw_command_palette, draw_help_popup};
