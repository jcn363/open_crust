use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::ThemeContext;
use crate::app::{App, Mode};

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
            let sep_line = Line::from(vec![Span::styled(
                "--- Chat Messages ---",
                Style::default().fg(Color::DarkGray),
            )]);
            all_items.push(ListItem::new(sep_line));
            all_items.extend(
                tab.messages
                    .iter()
                    .map(|m| ListItem::new(Line::from(Span::raw(m)))),
            );
        }
    } else {
        // Chat tab: just show messages
        if let Some(tab) = app.tabs.get(app.active_tab) {
            all_items.extend(
                tab.messages
                    .iter()
                    .map(|m| ListItem::new(Line::from(Span::raw(m)))),
            );
        }
    }

    let messages_list = List::new(all_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " {} ",
                app.tabs
                    .get(app.active_tab)
                    .map(|t| t.name.as_str())
                    .unwrap_or("Chat")
            ))
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().fg(theme.fg)),
    );
    f.render_widget(messages_list, area);
}

pub fn draw_input_area(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let input = Paragraph::new({
        let mut line = Line::from(app.input.clone());
        if let Some(ref ghost) = app.ghost_text {
            line.spans.push(Span::styled(
                ghost.clone(),
                Style::default().fg(Color::DarkGray),
            ));
        }
        line
    })
    .style(input_style_for_mode(app.mode, theme))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Input ")
            .border_style(input_border_style_for_mode(app.mode, theme)),
    );
    f.render_widget(input, area);

    // Cursor
    if let Mode::Insert = app.mode {
        let input_len = app.input.len() as u16;
        f.set_cursor_position((area.x + input_len + 1, area.y + 1));
    }
}

pub fn draw_sidebar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let sidebar_items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .map(|i| ListItem::new(Line::from(Span::raw(format!(" {} ", i)))))
        .collect();
    let sidebar_list = List::new(sidebar_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Files ")
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(sidebar_list, area);
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::ThemeContext;

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

    /// Verify sidebar items render correctly.
    #[test]
    fn test_sidebar_item_format() {
        let item = "main.rs";
        let formatted = format!(" {} ", item);
        assert_eq!(formatted, " main.rs ");
    }
}
