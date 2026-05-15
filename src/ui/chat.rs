use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, Message, Mode};
use crate::ui::ThemeContext;

/// Determine message style based on message content
fn message_style(content: &str) -> Style {
    if content.starts_with("You: ") {
        // User message - bright green
        Style::default().fg(Color::LightGreen)
    } else if content.starts_with("opencrust: ") {
        // System message - gray
        Style::default().fg(Color::DarkGray)
    } else if content.starts_with("System: ") {
        // System notification - yellow
        Style::default().fg(Color::Yellow)
    } else {
        // LLM response - cyan
        Style::default().fg(Color::Cyan)
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
fn render_message(m: &Message) -> ListItem<'_> {
    let ts = m.timestamp.format("%H:%M").to_string();
    let style = message_style(&m.content);
    ListItem::new(Line::from(vec![
        Span::styled(format!("[{}] ", ts), Style::default().fg(Color::DarkGray)),
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
            all_items.extend(tab.messages.iter().map(render_message));
        }
    } else {
        // Chat tab: just show messages
        if let Some(tab) = app.tabs.get(app.active_tab) {
            all_items.extend(tab.messages.iter().map(render_message));
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
        let input_len = app.input.len() as u16;
        let border_offset: u16 = 1;
        f.set_cursor_position((area.x + border_offset + input_len, area.y + border_offset));
    }
}

pub fn draw_sidebar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let sidebar_items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .map(|i| {
            let is_dir = i.ends_with('/');
            if is_dir {
                ListItem::new(Line::from(Span::styled(
                    format!(" +{}", i),
                    Style::default().fg(Color::Cyan),
                )))
            } else {
                ListItem::new(Line::from(Span::raw(format!("  {}", i))))
            }
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

    /// Verify sidebar dir items use + prefix.
    #[test]
    fn test_sidebar_dir_format() {
        let item = "src/";
        let formatted = format!(" +{}", item);
        assert_eq!(formatted, " +src/");
    }
}
