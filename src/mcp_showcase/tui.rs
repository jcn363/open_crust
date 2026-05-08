//! MCP Showcase TUI component for browsing/installing MCP servers
//! and browsing/executing MCP tools with an interactive form UI.
//! Uses ratatui for terminal rendering

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction},
};
use crate::mcp_showcase::McpServerInfo;
use crate::mcp_showcase::ToolInfo;
use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyModifiers};

/// Current view in the MCP Showcase
#[derive(Debug, Clone, PartialEq)]
pub enum ShowcaseView {
    ServerList,
    ToolList(String),
    ToolDetail(String, String),
    ToolResult(String, String, String),
}

/// Actions that can be returned by MCP Showcase TUI
#[allow(dead_code)]
pub enum McpShowcaseAction {
    None,
    ToggleServer(String),
    ExitMode,
    ShowTools(String),
    BackToServerList,
    #[allow(dead_code)]
    SelectTool(String, String),
    ExecuteTool(String, String),
    InputChanged(String, String),
    NextField,
    PrevField,
    ScrollResult(i32),
}

/// MCP Showcase TUI state
pub struct McpShowcaseUI {
    servers: Vec<McpServerInfo>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub visible_items: usize,
    pub view: ShowcaseView,
    pub tool_cache: HashMap<String, Vec<ToolInfo>>,
    pub input_fields: HashMap<String, String>,
    pub field_order: Vec<String>,
    pub selected_field: usize,
    pub result_scroll: usize,
}

impl McpShowcaseUI {
    /// Create a new MCP Showcase UI
    pub fn new(servers: Vec<McpServerInfo>) -> Self {
        Self {
            servers,
            selected_index: 0,
            scroll_offset: 0,
            visible_items: 10,
            view: ShowcaseView::ServerList,
            tool_cache: HashMap::new(),
            input_fields: HashMap::new(),
            field_order: Vec::new(),
            selected_field: 0,
            result_scroll: 0,
        }
    }

    /// Toggle the enabled status of a server by name
    pub fn toggle_server(&mut self, name: &str) {
        if let Some(server) = self.servers.iter_mut().find(|s| s.name == name) {
            server.enabled = !server.enabled;
        }
    }

    /// Set tools for a server (populates tool cache)
    pub fn set_tools_for_server(&mut self, server_name: &str, tools: Vec<ToolInfo>) {
        self.tool_cache.insert(server_name.to_string(), tools);
    }

    /// Navigate to tool list view for a server
    pub fn navigate_to_tool_list(&mut self, server_name: &str) {
        self.view = ShowcaseView::ToolList(server_name.to_string());
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Navigate to tool detail view for a specific tool
    pub fn navigate_to_tool_detail(&mut self, server_name: &str, tool_name: &str) {
        self.view = ShowcaseView::ToolDetail(server_name.to_string(), tool_name.to_string());
        self.selected_field = 0;
        self.result_scroll = 0;
        self.input_fields.clear();
        self.field_order.clear();

        if let Some(tools) = self.tool_cache.get(server_name) {
            if let Some(tool) = tools.iter().find(|t| t.name == tool_name) {
                if let Some(properties) = tool.input_schema.get("properties").and_then(|p| p.as_object()) {
                    for key in properties.keys() {
                        self.input_fields.insert(key.clone(), String::new());
                        self.field_order.push(key.clone());
                    }
                }
            }
        }
    }

    /// Navigate to result view
    pub fn navigate_to_result(&mut self, server_name: &str, tool_name: &str, result: &str) {
        self.view = ShowcaseView::ToolResult(
            server_name.to_string(),
            tool_name.to_string(),
            result.to_string(),
        );
        self.result_scroll = 0;
    }

    /// Go back one level in the view stack
    pub fn go_back(&mut self) {
        match &self.view {
            ShowcaseView::ToolResult(server_name, tool_name, _) => {
                self.view = ShowcaseView::ToolDetail(server_name.clone(), tool_name.clone());
            }
            ShowcaseView::ToolDetail(server_name, _) => {
                self.view = ShowcaseView::ToolList(server_name.clone());
            }
            ShowcaseView::ToolList(_) => {
                self.view = ShowcaseView::ServerList;
            }
            ShowcaseView::ServerList => {}
        }
    }

    /// Update an input field value
    pub fn update_input_field(&mut self, field_name: &str, value: String) {
        self.input_fields.insert(field_name.to_string(), value);
    }

    /// Get the current field name at the selected index (for ToolDetail)
    fn current_field_name(&self) -> Option<&str> {
        self.field_order.get(self.selected_field).map(|s| s.as_str())
    }

    /// Build arguments JSON from input fields based on schema
    #[allow(unused_variables)]
    pub fn build_arguments_json(&self, server_name: &str, tool_name: &str) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.input_fields {
            if !value.is_empty() {
                map.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }
        serde_json::Value::Object(map)
    }

    /// Handle key events for navigation
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> McpShowcaseAction {
        match &self.view.clone() {
            ShowcaseView::ServerList => self.handle_server_list_key(key),
            ShowcaseView::ToolList(server_name) => self.handle_tool_list_key(key, server_name),
            ShowcaseView::ToolDetail(server_name, tool_name) => {
                self.handle_tool_detail_key(key, modifiers, server_name, tool_name)
            }
            ShowcaseView::ToolResult(_, _, _) => self.handle_result_key(key),
        }
    }

    fn handle_server_list_key(&mut self, key: KeyCode) -> McpShowcaseAction {
        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Down => {
                if self.selected_index < self.servers.len().saturating_sub(1) {
                    self.selected_index += 1;
                    if self.selected_index >= self.scroll_offset + self.visible_items {
                        self.scroll_offset = self.selected_index - self.visible_items + 1;
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Enter => {
                if let Some(server) = self.servers.get(self.selected_index) {
                    return McpShowcaseAction::ToggleServer(server.name.clone());
                }
                McpShowcaseAction::None
            }
            KeyCode::Right => {
                if let Some(server) = self.servers.get(self.selected_index) {
                    if server.enabled {
                        return McpShowcaseAction::ShowTools(server.name.clone());
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Esc => McpShowcaseAction::ExitMode,
            _ => McpShowcaseAction::None,
        }
    }

    fn handle_tool_list_key(&mut self, key: KeyCode, server_name: &str) -> McpShowcaseAction {
        let tools = self.tool_cache.get(server_name);
        let tool_count = tools.map(|t| t.len()).unwrap_or(0);

        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Down => {
                if tool_count > 0 && self.selected_index < tool_count.saturating_sub(1) {
                    self.selected_index += 1;
                    if self.selected_index >= self.scroll_offset + self.visible_items {
                        self.scroll_offset = self.selected_index - self.visible_items + 1;
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Enter => {
                if let Some(tools) = self.tool_cache.get(server_name) {
                    if let Some(tool) = tools.get(self.selected_index) {
                        let tool_name = tool.name.clone();
                        self.navigate_to_tool_detail(server_name, &tool_name);
                        return McpShowcaseAction::SelectTool(server_name.to_string(), tool_name);
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Left | KeyCode::Esc => {
                self.go_back();
                McpShowcaseAction::BackToServerList
            }
            _ => McpShowcaseAction::None,
        }
    }

    fn handle_tool_detail_key(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        server_name: &str,
        tool_name: &str,
    ) -> McpShowcaseAction {
        match key {
            KeyCode::Tab => {
                if !self.field_order.is_empty() {
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        if self.selected_field > 0 {
                            self.selected_field -= 1;
                        }
                        return McpShowcaseAction::PrevField;
                    } else {
                        self.selected_field = (self.selected_field + 1) % self.field_order.len();
                        return McpShowcaseAction::NextField;
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Up => {
                if self.selected_field > 0 {
                    self.selected_field -= 1;
                }
                McpShowcaseAction::PrevField
            }
            KeyCode::Down => {
                if self.selected_field + 1 < self.field_order.len() {
                    self.selected_field += 1;
                }
                McpShowcaseAction::NextField
            }
            KeyCode::Enter if modifiers.contains(KeyModifiers::CONTROL) => {
                McpShowcaseAction::ExecuteTool(server_name.to_string(), tool_name.to_string())
            }
            KeyCode::Esc => {
                self.go_back();
                McpShowcaseAction::BackToServerList
            }
            KeyCode::Backspace => {
                if let Some(field_name) = self.current_field_name().map(|s| s.to_string()) {
                    if let Some(current) = self.input_fields.get(&field_name) {
                        let mut new_val = current.clone();
                        new_val.pop();
                        self.input_fields.insert(field_name.clone(), new_val.clone());
                        return McpShowcaseAction::InputChanged(field_name, new_val);
                    }
                }
                McpShowcaseAction::None
            }
            KeyCode::Char(c) => {
                if let Some(field_name) = self.current_field_name().map(|s| s.to_string()) {
                    let current = self.input_fields.get(&field_name).cloned().unwrap_or_default();
                    let mut new_val = current;
                    new_val.push(c);
                    self.input_fields.insert(field_name.clone(), new_val.clone());
                    return McpShowcaseAction::InputChanged(field_name, new_val);
                }
                McpShowcaseAction::None
            }
            _ => McpShowcaseAction::None,
        }
    }

    fn handle_result_key(&mut self, key: KeyCode) -> McpShowcaseAction {
        match key {
            KeyCode::Up => {
                self.result_scroll = self.result_scroll.saturating_sub(1);
                McpShowcaseAction::ScrollResult(-1)
            }
            KeyCode::Down => {
                self.result_scroll = self.result_scroll.saturating_add(1);
                McpShowcaseAction::ScrollResult(1)
            }
            KeyCode::Esc => {
                self.go_back();
                McpShowcaseAction::BackToServerList
            }
            _ => McpShowcaseAction::None,
        }
    }

    /// Render the MCP Showcase TUI
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match &self.view.clone() {
            ShowcaseView::ServerList => self.render_server_list(f, area),
            ShowcaseView::ToolList(server_name) => self.render_tool_list(f, area, server_name),
            ShowcaseView::ToolDetail(server_name, tool_name) => {
                self.render_tool_detail(f, area, server_name, tool_name)
            }
            ShowcaseView::ToolResult(_, _, result) => self.render_tool_result(f, area, result),
        }
    }

    /// Render the server list view
    fn render_server_list(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Paragraph::new("MCP Server Showcase")
            .block(Block::default().borders(Borders::ALL).title("MCP Servers"))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title, chunks[0]);

        let items: Vec<ListItem> = self.servers
            .iter()
            .map(|server| {
                let checkbox = if server.enabled { "[✓]" } else { "[ ]" };
                let status = if server.installed {
                    if server.enabled { "Enabled" } else { "Disabled" }
                } else {
                    "Not Installed"
                };
                let line = format!("{} {} - {} [{}]", checkbox, server.name, server.description, status);
                ListItem::new(line)
            })
            .collect();

        self.visible_items = chunks[1].height as usize - 2;
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Servers"))
            .highlight_style(Style::default().bg(Color::DarkGray));
        let mut state = ListState::default().with_selected(Some(self.selected_index));
        state.select(Some(self.selected_index));
        f.render_stateful_widget(list, chunks[1], &mut state);

        let help = Paragraph::new("↑/↓: Navigate | Enter: Toggle | →: Browse tools | Esc: Close")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[2]);
    }

    /// Render the tool list view for a server
    fn render_tool_list(&mut self, f: &mut Frame, area: Rect, server_name: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let title = format!("Tools for {}", server_name);
        let title_block = Paragraph::new(title.as_str())
            .block(Block::default().borders(Borders::ALL).title("MCP Tools"))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title_block, chunks[0]);

        let tools = self.tool_cache.get(server_name).cloned().unwrap_or_default();

        let items: Vec<ListItem> = tools
            .iter()
            .map(|tool| {
                let desc = if tool.description.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", tool.description)
                };
                ListItem::new(format!("{} {}", tool.name, desc))
            })
            .collect();

        self.visible_items = chunks[1].height as usize - 2;
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!("{} ({} tools)", server_name, tools.len())))
            .highlight_style(Style::default().bg(Color::DarkGray));
        let mut state = ListState::default().with_selected(Some(self.selected_index));
        state.select(Some(self.selected_index));
        f.render_stateful_widget(list, chunks[1], &mut state);

        let help = Paragraph::new("↑/↓: Navigate | Enter: View tool | ←/Esc: Back")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[2]);
    }

    /// Render the tool detail view with input form
    fn render_tool_detail(&mut self, f: &mut Frame, area: Rect, server_name: &str, tool_name: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let tool = self.tool_cache.get(server_name)
            .and_then(|tools| tools.iter().find(|t| t.name == tool_name));

        let (tool_desc, schema) = match tool {
            Some(t) => (t.description.clone(), t.input_schema.clone()),
            None => (String::new(), serde_json::json!({})),
        };

        let mut info_text = format!("Tool: {}  (server: {})\n", tool_name, server_name);
        if !tool_desc.is_empty() {
            info_text.push_str(&format!("Description: {}\n", tool_desc));
        }
        let info_block = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Tool Info"))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(info_block, chunks[0]);

        let properties = schema.get("properties").and_then(|p| p.as_object()).cloned().unwrap_or_default();
        let required = schema.get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_default();

        let mut form_lines: Vec<(String, String, bool)> = Vec::new();
        for (i, key) in self.field_order.iter().enumerate() {
            let prop = properties.get(key);
            let type_str = prop.and_then(|p| p.get("type")).and_then(|t| t.as_str()).unwrap_or("string");
            let prop_desc = prop.and_then(|p| p.get("description")).and_then(|d| d.as_str()).unwrap_or("");
            let is_required = required.contains(key);
            let required_mark = if is_required { " *" } else { "" };
            let label = format!("{}{} ({}): {}", key, required_mark, type_str, prop_desc);
            let value = self.input_fields.get(key).cloned().unwrap_or_default();
            let is_selected = i == self.selected_field;
            form_lines.push((label, value, is_selected));
        }

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
            ])
            .split(chunks[1]);

        let mut form_text = String::new();
        for (label, value, is_selected) in &form_lines {
            let cursor = if *is_selected { " <--" } else { "" };
            form_text.push_str(&format!("{} {}\n  {}{}\n\n", label, cursor, value, cursor));
        }

        if form_lines.is_empty() {
            form_text = "  No input parameters required.".to_string();
        }

        let form_block = Paragraph::new(form_text)
            .block(Block::default().borders(Borders::ALL).title("Input Parameters"))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        f.render_widget(form_block, form_chunks[0]);

        let help = Paragraph::new(
            "↑/↓: Select field | Tab: Next field | Type: Edit field | Ctrl+Enter: Execute | Esc: Back"
        )
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[2]);
    }

    /// Render the tool execution result view
    fn render_tool_result(&mut self, f: &mut Frame, area: Rect, result: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let header = Paragraph::new("Tool Execution Result")
            .block(Block::default().borders(Borders::ALL).title("Result"))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, chunks[0]);

        let result_block = Paragraph::new(result.to_string())
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .style(Style::default().fg(Color::White))
            .scroll((self.result_scroll as u16, 0))
            .wrap(Wrap { trim: false });
        f.render_widget(result_block, chunks[1]);

        let help = Paragraph::new("↑/↓: Scroll | Esc: Back to tool detail")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ui() -> McpShowcaseUI {
        let servers = vec![
            McpServerInfo {
                name: "server1".to_string(),
                description: "Test server 1".to_string(),
                installed: true,
                enabled: true,
                command: "cmd1".to_string(),
            },
            McpServerInfo {
                name: "server2".to_string(),
                description: "Test server 2".to_string(),
                installed: true,
                enabled: false,
                command: "cmd2".to_string(),
            },
            McpServerInfo {
                name: "server3".to_string(),
                description: "Test server 3".to_string(),
                installed: false,
                enabled: false,
                command: "cmd3".to_string(),
            },
        ];
        McpShowcaseUI::new(servers)
    }

    fn create_test_tools() -> Vec<ToolInfo> {
        vec![
            ToolInfo {
                name: "tool1".to_string(),
                description: "Test tool 1".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Max results"
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolInfo {
                name: "tool2".to_string(),
                description: "Test tool 2".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path"
                        }
                    }
                }),
            },
        ]
    }

    // Existing tests - adapted for new handle_key signature

    #[test]
    fn test_handle_key_up() {
        let mut ui = create_test_ui();
        ui.selected_index = 1;

        let action = ui.handle_key(KeyCode::Up, KeyModifiers::NONE);

        assert_eq!(ui.selected_index, 0);
        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_handle_key_down() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Down, KeyModifiers::NONE);

        assert_eq!(ui.selected_index, 1);
        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_handle_key_down_at_bottom() {
        let mut ui = create_test_ui();
        ui.selected_index = 2;

        let action = ui.handle_key(KeyCode::Down, KeyModifiers::NONE);

        assert_eq!(ui.selected_index, 2);
        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_handle_key_enter_toggle() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        assert!(matches!(action, McpShowcaseAction::ToggleServer(ref name) if name == "server1"));
    }

    #[test]
    fn test_handle_key_esc() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Esc, KeyModifiers::NONE);

        assert!(matches!(action, McpShowcaseAction::ExitMode));
    }

    #[test]
    fn test_handle_key_unknown() {
        let mut ui = create_test_ui();

        let action = ui.handle_key(KeyCode::Char('x'), KeyModifiers::NONE);

        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_toggle_server() {
        let mut ui = create_test_ui();

        assert!(ui.servers[0].enabled);

        ui.toggle_server("server1");

        assert!(!ui.servers[0].enabled);

        ui.toggle_server("server1");

        assert!(ui.servers[0].enabled);
    }

    #[test]
    fn test_toggle_nonexistent_server() {
        let mut ui = create_test_ui();

        ui.toggle_server("nonexistent");

        assert!(ui.servers[0].enabled);
        assert!(!ui.servers[1].enabled);
    }

    #[test]
    fn test_render_does_not_panic() {
        let ui = create_test_ui();

        assert_eq!(ui.servers.len(), 3);
    }

    // New tests

    #[test]
    fn test_view_navigation() {
        let mut ui = create_test_ui();

        // Start in ServerList
        assert_eq!(ui.view, ShowcaseView::ServerList);

        // Navigate to ToolList
        let tools = create_test_tools();
        ui.set_tools_for_server("server1", tools);
        ui.navigate_to_tool_list("server1");
        assert_eq!(ui.view, ShowcaseView::ToolList("server1".to_string()));

        // Right arrow in ServerList should show tools
        // Navigate to ToolDetail directly
        ui.navigate_to_tool_detail("server1", "tool1");
        assert_eq!(ui.view, ShowcaseView::ToolDetail("server1".to_string(), "tool1".to_string()));

        // Go back to ToolList
        ui.go_back();
        assert_eq!(ui.view, ShowcaseView::ToolList("server1".to_string()));

        // Go back to ServerList
        ui.go_back();
        assert_eq!(ui.view, ShowcaseView::ServerList);

        // Esc from ServerList should return ExitMode
        let action = ui.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::ExitMode));
    }

    #[test]
    fn test_input_field_management() {
        let mut ui = create_test_ui();
        let tools = create_test_tools();
        ui.set_tools_for_server("server1", tools);
        ui.navigate_to_tool_detail("server1", "tool1");

        // Check that input fields were initialized
        assert!(ui.input_fields.contains_key("query"));
        assert!(ui.input_fields.contains_key("limit"));
        assert_eq!(ui.input_fields.get("query").unwrap(), "");
        assert_eq!(ui.field_order.len(), 2);

        // Update a field
        ui.update_input_field("query", "test_value".to_string());
        assert_eq!(ui.input_fields.get("query").unwrap(), "test_value");

        // Check input field via Char key
        let action = ui.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::InputChanged(ref field, ref val)
            if field == "query" && val == "h"));

        // Tab to next field
        let action = ui.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::NextField));
        assert_eq!(ui.selected_field, 1);

        // Backspace on current field
        ui.input_fields.insert("limit".to_string(), "10".to_string());
        let action = ui.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::InputChanged(ref field, ref val)
            if field == "limit" && val == "1"));
    }

    #[test]
    fn test_tool_cache() {
        let mut ui = create_test_ui();
        let tools = create_test_tools();

        // Set tools for a server
        ui.set_tools_for_server("server1", tools.clone());
        assert_eq!(ui.tool_cache.len(), 1);

        let cached = ui.tool_cache.get("server1");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 2);
        assert_eq!(cached.unwrap()[0].name, "tool1");

        // Set tools for another server
        let more_tools = vec![ToolInfo {
            name: "tool3".to_string(),
            description: "Test tool 3".to_string(),
            input_schema: serde_json::json!({}),
        }];
        ui.set_tools_for_server("server2", more_tools);
        assert_eq!(ui.tool_cache.len(), 2);
    }

    #[test]
    fn test_show_tools_action() {
        let mut ui = create_test_ui();

        // Right arrow on enabled server should return ShowTools
        let action = ui.handle_key(KeyCode::Right, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::ShowTools(ref name) if name == "server1"));

        // Move to disabled server
        ui.selected_index = 1;
        let action = ui.handle_key(KeyCode::Right, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::None));
    }

    #[test]
    fn test_result_navigation() {
        let mut ui = create_test_ui();

        // Navigate to result view
        ui.navigate_to_result("server1", "tool1", "test output");
        assert_eq!(ui.view, ShowcaseView::ToolResult("server1".to_string(), "tool1".to_string(), "test output".to_string()));

        // Scroll down
        let action = ui.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::ScrollResult(1)));
        assert_eq!(ui.result_scroll, 1);

        // Scroll up
        let action = ui.handle_key(KeyCode::Up, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::ScrollResult(-1)));
        assert_eq!(ui.result_scroll, 0);

        // Esc goes back to ToolDetail
        let action = ui.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::BackToServerList));
        assert_eq!(ui.view, ShowcaseView::ToolDetail("server1".to_string(), "tool1".to_string()));
    }

    #[test]
    fn test_navigate_to_result_error() {
        let mut ui = create_test_ui();

        // Navigate to result with error message
        ui.navigate_to_result("server1", "tool1", "Error: Something went wrong");
        assert_eq!(ui.view, ShowcaseView::ToolResult("server1".to_string(), "tool1".to_string(), "Error: Something went wrong".to_string()));

        // Verify we can scroll the error text
        let action = ui.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::ScrollResult(1)));
    }

    #[test]
    fn test_input_field_field_order() {
        let mut ui = create_test_ui();
        let tools = create_test_tools();
        ui.set_tools_for_server("server1", tools);
        ui.navigate_to_tool_detail("server1", "tool1");

        // Field order should match schema properties order
        assert_eq!(ui.field_order.len(), 2);
        assert_eq!(ui.field_order[0], "query");
        assert_eq!(ui.field_order[1], "limit");

        // Tool2 has only "path"
        ui.navigate_to_tool_detail("server1", "tool2");
        assert_eq!(ui.field_order.len(), 1);
        assert_eq!(ui.field_order[0], "path");
    }

    #[test]
    fn test_go_back_from_all_views() {
        let mut ui = create_test_ui();
        let tools = create_test_tools();
        ui.set_tools_for_server("server1", tools);

        // ServerList -> go_back stays
        assert_eq!(ui.view, ShowcaseView::ServerList);
        ui.go_back();
        assert_eq!(ui.view, ShowcaseView::ServerList);

        // ToolList -> go_back
        ui.navigate_to_tool_list("server1");
        assert_eq!(ui.view, ShowcaseView::ToolList("server1".to_string()));
        ui.go_back();
        assert_eq!(ui.view, ShowcaseView::ServerList);

        // Detail -> go_back
        ui.navigate_to_tool_detail("server1", "tool1");
        assert_eq!(ui.view, ShowcaseView::ToolDetail("server1".to_string(), "tool1".to_string()));
        ui.go_back();
        assert_eq!(ui.view, ShowcaseView::ToolList("server1".to_string()));

        // Result -> go_back
        ui.navigate_to_result("server1", "tool1", "result");
        ui.go_back();
        assert_eq!(ui.view, ShowcaseView::ToolDetail("server1".to_string(), "tool1".to_string()));
    }

    #[test]
    fn test_build_arguments_json() {
        let mut ui = create_test_ui();
        let tools = create_test_tools();
        ui.set_tools_for_server("server1", tools);
        ui.navigate_to_tool_detail("server1", "tool1");

        ui.update_input_field("query", "search terms".to_string());
        ui.update_input_field("limit", "5".to_string());

        let args = ui.build_arguments_json("server1", "tool1");
        assert_eq!(args.get("query").and_then(|v| v.as_str()), Some("search terms"));
        assert_eq!(args.get("limit").and_then(|v| v.as_str()), Some("5"));
    }

    #[test]
    fn test_tool_list_enter_selects_tool() {
        let mut ui = create_test_ui();
        let tools = create_test_tools();
        ui.set_tools_for_server("server1", tools);
        ui.navigate_to_tool_list("server1");

        // Enter on first tool
        let action = ui.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(matches!(action, McpShowcaseAction::SelectTool(ref server, ref tool)
            if server == "server1" && tool == "tool1"));

        // View should have changed to tool detail
        assert_eq!(ui.view, ShowcaseView::ToolDetail("server1".to_string(), "tool1".to_string()));
    }

    #[test]
    fn test_execute_tool_action() {
        let mut ui = create_test_ui();
        let tools = create_test_tools();
        ui.set_tools_for_server("server1", tools);
        ui.navigate_to_tool_detail("server1", "tool1");

        // Ctrl+Enter should return ExecuteTool
        let action = ui.handle_key(KeyCode::Enter, KeyModifiers::CONTROL);
        assert!(matches!(action, McpShowcaseAction::ExecuteTool(ref server, ref tool)
            if server == "server1" && tool == "tool1"));
    }
}
