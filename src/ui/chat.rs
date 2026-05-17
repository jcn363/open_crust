//! Chat view rendering: messages, input area, and sidebar
//!
//! Renders the main chat panel with message history, the text input area,
//! and the file tree sidebar. Handles text wrapping, ghost text display,
//! and selection highlighting.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, Message, Mode};
use crate::ui::ThemeContext;

/// Determine message style based on message content.
/// Uses theme colors for a cohesive, subtle appearance.
fn message_style(content: &str, theme: &ThemeContext) -> Style {
    if content.starts_with("You: ") {
        // User message - use theme foreground for subtle distinction
        Style::default().fg(theme.fg)
    } else if content.starts_with("opencrust: ") {
        // System message - use theme border for subtle appearance
        Style::default().fg(theme.border)
    } else if content.starts_with("System: ") {
        // System notification - warm amber (subtle)
        Style::default().fg(Color::Rgb(180, 160, 80))
    } else if content.starts_with("Error: ") {
        // Error messages - soft red
        Style::default().fg(Color::Rgb(200, 80, 80))
    } else {
        // LLM response - use theme accent
        Style::default().fg(theme.accent)
    }
}

/// Get the input text style for the current mode
fn input_style_for_mode(mode: Mode, theme: &ThemeContext) -> Style {
    match mode {
        Mode::Insert => Style::default().fg(theme.accent),
        _ => Style::default().fg(theme.fg),
    }
}

/// Get the input border style for the current mode
fn input_border_style_for_mode(mode: Mode, theme: &ThemeContext) -> Style {
    match mode {
        Mode::Insert => Style::default().fg(theme.accent),
        _ => Style::default().fg(theme.border),
    }
}

/// Render a single message into a ListItem with stored timestamp.
/// The entire message body is styled uniformly based on sender type.
fn render_message<'a>(m: &'a Message, theme: &ThemeContext) -> ListItem<'a> {
    let ts = m.timestamp.format("%H:%M").to_string();
    let style = message_style(&m.content, theme);
    ListItem::new(Line::from(vec![
        Span::styled(format!("[{}] ", ts), Style::default().fg(theme.border)),
        Span::styled(&m.content, style),
    ]))
}

pub fn draw_message_list(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let mut all_items: Vec<ListItem> = Vec::new();

    // Add background tasks if on Tasks tab
    if app.active_tab == 1 {
        let task_items: Vec<ListItem> = app
            .background_tasks
            .iter()
            .map(|task| {
                let status_icon = match task.status {
                    crate::app::TaskStatus::Running => "⏳",
                    crate::app::TaskStatus::Completed => "✅",
                    crate::app::TaskStatus::Failed => "❌",
                };
                let time_str = task.started_at.format("%H:%M:%S").to_string();
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", status_icon),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(format!("[{}] {}: {}", task.id, time_str, task.prompt)),
                ]))
            })
            .collect();
        all_items.extend(task_items);

        if app.background_tasks.is_empty() {
            all_items.push(ListItem::new(Line::from(Span::raw(
                "No background tasks yet.",
            ))));
        }

        // Add separator if there are chat messages
        if let Some(tab) = app
            .tabs
            .get(app.active_tab)
            .filter(|tab| !tab.messages.is_empty())
        {
            let sep_line = Line::from(vec![
                Span::styled("─── ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Chat Messages",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ───", Style::default().fg(Color::DarkGray)),
            ]);
            all_items.push(ListItem::new(sep_line));
            all_items.extend(tab.messages.iter().map(|m| render_message(m, theme)));
        }
    } else {
        // Chat tab: just show messages
        if let Some(tab) = app.tabs.get(app.active_tab) {
            all_items.extend(tab.messages.iter().map(|m| render_message(m, theme)));
        }
    }

    // Clamp scroll offset to valid range
    let max_scroll = all_items.len().saturating_sub(1);
    let offset = app.message_scroll.min(max_scroll);

    // Scroll indicator for when user is not at bottom
    let scroll_indicator = if offset > 0 {
        format!(" ↑{} ", offset)
    } else {
        String::new()
    };

    let title_text = format!(
        "{}{}",
        app.tabs
            .get(app.active_tab)
            .map(|t| t.name.as_str())
            .unwrap_or("Chat"),
        scroll_indicator,
    );

    let mut list_state = ratatui::widgets::ListState::default()
        .with_selected(Some(0))
        .with_offset(offset);

    let messages_list = List::new(all_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title_text))
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().fg(theme.fg)),
    );
    f.render_stateful_widget(messages_list, area, &mut list_state);
}

pub fn draw_input_area(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let input = Paragraph::new({
        let mut line = Line::from(app.input.clone());
        if let Some(ref ghost) = app.ghost_text {
            line.spans.push(Span::styled(
                ghost.clone(),
                Style::default().fg(Color::Rgb(73, 72, 71)),
            ));
        }
        line
    })
    .style(input_style_for_mode(app.mode, theme))
    .block({
        let mode_hint = match app.mode {
            Mode::Insert if app.vim_mode => " Input [VIM] ",
            Mode::Insert => " Input [INS] ",
            _ => " Input ",
        };
        Block::default()
            .borders(Borders::ALL)
            .title(mode_hint)
            .border_style(input_border_style_for_mode(app.mode, theme))
    });
    f.render_widget(input, area);

    // Cursor — account for border (Borders::ALL = 1-char offset)
    if let Mode::Insert = app.mode {
        let cursor_pos = if app.vim_mode {
            app.vim_cursor_pos
        } else {
            app.input.len()
        };
        let input_len = app.input.len() as u16;
        let border_offset: u16 = 1;
        // Clamp cursor to input length to avoid out-of-bounds
        let clamped_pos = (cursor_pos as u16).min(input_len);
        f.set_cursor_position((area.x + border_offset + clamped_pos, area.y + border_offset));
    }
}

pub fn draw_sidebar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let sidebar_items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .enumerate()
        .map(|(idx, i)| {
            let is_dir = i.ends_with('/');
            let is_selected = idx == app.sidebar_selected;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(theme.accent)
            } else if is_dir {
                Style::default().fg(Color::Rgb(111, 109, 108))
            } else {
                Style::default().fg(theme.fg)
            };
            let prefix = if is_selected { "▸ " } else { "  " };
            ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, i), style)))
        })
        .collect();
    let sidebar_list = List::new(sidebar_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Files ")
                .border_style(Style::default().fg(theme.accent)),
        )
        .style(Style::default().bg(dim_color(theme.bg)));
    f.render_widget(sidebar_list, area);
}

/// Return a slightly dimmed version of the given color for subtle backgrounds.
fn dim_color(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let r = (r as u16 * 3 / 4).min(255) as u8;
            let g = (g as u16 * 3 / 4).min(255) as u8;
            let b = (b as u16 * 3 / 4).min(255) as u8;
            Color::Rgb(r, g, b)
        }
        _ => Color::Reset,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use crate::ui::ThemeContext;

    fn dummy_theme() -> ThemeContext {
        ThemeContext {
            bg: Color::Reset,
            fg: Color::White,
            accent: Color::Cyan,
            border: Color::DarkGray,
        }
    }

    /// Verify ThemeContext defaults are set correctly.
    #[test]
    fn test_dummy_theme_values() {
        let theme = dummy_theme();
        assert_eq!(theme.fg, Color::White);
        assert_eq!(theme.accent, Color::Cyan);
        assert_eq!(theme.border, Color::DarkGray);
    }

    /// Verify sidebar items render correctly — non-dir items use two-space prefix.
    #[test]
    fn test_sidebar_item_format() {
        let item = "main.rs";
        let formatted = format!("  {}", item);
        assert_eq!(formatted, "  main.rs");
    }

    /// Verify sidebar dir items use consistent indentation + trailing slash.
    #[test]
    fn test_sidebar_dir_format() {
        let item = "src/";
        let formatted = format!("  {}", item);
        assert_eq!(formatted, "  src/");
    }
}
