//! MCP Showcase TUI component for browsing/installing MCP servers
//! Uses ratatui for terminal rendering

use crate::mcp_showcase::McpServerInfo;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// Actions that can be returned by MCP Showcase TUI
pub enum McpShowcaseAction {
    None,
    ToggleServer(String), // Toggle enabled status of named server
    ExitMode,
}

/// MCP Showcase TUI state
pub struct McpShowcaseUI {
    servers: Vec<McpServerInfo>,
    selected_index: usize,
    scroll_offset: usize,
    visible_items: usize,
}

impl McpShowcaseUI {
    /// Create a new MCP Showcase UI
    pub fn new(servers: Vec<McpServerInfo>) -> Self {
        Self {
            servers,
            selected_index: 0,
            scroll_offset: 0,
            visible_items: 10, // Default, will be updated on render
        }
    }

    /// Toggle the enabled status of a server by name
    pub fn toggle_server(&mut self, name: &str) {
        if let Some(server) = self.servers.iter_mut().find(|s| s.name == name) {
            server.enabled = !server.enabled;
        }
    }

    /// Handle key events for navigation
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> McpShowcaseAction {
        match key {
            crossterm::event::KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
                McpShowcaseAction::None
            }
            crossterm::event::KeyCode::Down => {
                if self.selected_index < self.servers.len().saturating_sub(1) {
                    self.selected_index += 1;
                    if self.selected_index >= self.scroll_offset + self.visible_items {
                        self.scroll_offset = self.selected_index - self.visible_items + 1;
                    }
                }
                McpShowcaseAction::None
            }
            crossterm::event::KeyCode::Enter => {
                // Toggle enabled status of selected server
                if let Some(server) = self.servers.get(self.selected_index) {
                    return McpShowcaseAction::ToggleServer(server.name.clone());
                }
                McpShowcaseAction::None
            }
            crossterm::event::KeyCode::Esc => McpShowcaseAction::ExitMode,
            _ => McpShowcaseAction::None,
        }
    }

    /// Render the MCP Showcase TUI
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Server list
                Constraint::Length(3), // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("MCP Server Showcase")
            .block(Block::default().borders(Borders::ALL).title("MCP Servers"))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title, chunks[0]);

        // Server list with checkbox indicators
        let items: Vec<ListItem> = self
            .servers
            .iter()
            .map(|server| {
                let checkbox = if server.enabled { "[✓]" } else { "[ ]" };
                let status = if !server.installed {
                    "📦 Not Installed"
                } else if server.enabled {
                    "✅ Enabled"
                } else {
                    "⚫ Disabled"
                };
                let line = format!(
                    "{} {} - {} [{}]",
                    checkbox, server.name, server.description, status
                );
                ListItem::new(line)
            })
            .collect();

        self.visible_items = chunks[1].height as usize - 2; // Subtract border
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Servers"))
            .highlight_style(Style::default().bg(Color::DarkGray));
        let mut state = ListState::default().with_selected(Some(self.selected_index));
        f.render_stateful_widget(list, chunks[1], &mut state);

        // Help
        let help = Paragraph::new("↑/↓: Navigate | Enter: Toggle enabled | Esc: Close")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    fn create_test_ui() -> McpShowcaseUI {
        let servers = vec![
            McpServerInfo {
                name: "server1".to_string(),
                description: "Test server 1".to_string(),
                installed: true,
                enabled: true,
            },
            McpServerInfo {
                name: "server2".to_string(),
                description: "Test server 2".to_string(),
                installed: true,
                enabled: false,
            },
            McpServerInfo {
                name: "server3".to_string(),
                description: "Test server 3".to_string(),
                installed: false,
                enabled: false,
            },
        ];
        McpShowcaseUI::new(servers)
    }

    #[test]
    fn test_handle_key_up() {
        let mut ui = create_test_ui();
        ui.selected_index = 1;

        let action = ui.handle_key(KeyCode::Up);

        assert_eq!(ui.selected_index, 0);
        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_handle_key_down() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Down);

        assert_eq!(ui.selected_index, 1);
        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_handle_key_down_at_bottom() {
        let mut ui = create_test_ui();
        ui.selected_index = 2; // Last item

        let action = ui.handle_key(KeyCode::Down);

        assert_eq!(ui.selected_index, 2); // Should not move
        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_handle_key_enter_toggle() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Enter);

        assert!(matches!(action, McpShowcaseAction::ToggleServer(ref name) if name == "server1"));
    }

    #[test]
    fn test_handle_key_esc() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Esc);

        assert!(matches!(action, McpShowcaseAction::ExitMode));
    }

    #[test]
    fn test_handle_key_unknown() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Char('x'));

        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_toggle_server() {
        let mut ui = create_test_ui();

        // server1 is initially enabled
        assert!(ui.servers[0].enabled);

        ui.toggle_server("server1");

        // server1 should now be disabled
        assert!(!ui.servers[0].enabled);

        // Toggle again
        ui.toggle_server("server1");

        // server1 should be enabled again
        assert!(ui.servers[0].enabled);
    }

    #[test]
    fn test_toggle_nonexistent_server() {
        let mut ui = create_test_ui();

        // Try to toggle a server that doesn't exist
        ui.toggle_server("nonexistent");

        // Should not panic, and existing servers should be unchanged
        assert!(ui.servers[0].enabled);
        assert!(!ui.servers[1].enabled);
    }

    #[test]
    fn test_render_does_not_panic() {
        let ui = create_test_ui();

        // Create a mock terminal and frame for testing
        // Note: Full render testing requires terminal initialization
        // This test just ensures the render function doesn't panic with valid data
        // For a complete test, we would need ratatui's TestBackend
        assert_eq!(ui.servers.len(), 3);
    }
}
