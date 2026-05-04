use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &App) {
    let theme = app.config.theme.as_ref();
    let bg_color = parse_color(theme.map(|t| t.background.as_str()).unwrap_or("#1e1e1e"));
    let fg_color = parse_color(theme.map(|t| t.foreground.as_str()).unwrap_or("#ffffff"));
    let accent_color = parse_color(theme.map(|t| t.accent.as_str()).unwrap_or("#007acc"));
    let border_color = parse_color(theme.map(|t| t.border.as_str()).unwrap_or("#333333"));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ].as_ref())
        .split(f.area());

    // Background
    let bg_block = Block::default().style(Style::default().bg(bg_color));
    f.render_widget(bg_block, f.area());

    // Messages
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| ListItem::new(Line::from(Span::raw(m))))
        .collect();
    let messages_list = List::new(messages)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Conversation ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().fg(fg_color)));
    f.render_widget(messages_list, chunks[0]);

    // Input Box
    let input = Paragraph::new(app.input.as_str())
        .style(match app.mode {
            Mode::Normal => Style::default().fg(fg_color),
            Mode::Insert => Style::default().fg(accent_color),
        })
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Input ")
            .border_style(match app.mode {
                Mode::Normal => Style::default().fg(border_color),
                Mode::Insert => Style::default().fg(accent_color),
            }));
    f.render_widget(input, chunks[1]);
    
    // Status
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
    };
    let status = Paragraph::new(format!("-- {} --", mode_str))
        .style(Style::default().bg(accent_color).fg(fg_color));
    f.render_widget(status, chunks[2]);
    
    // Cursor handling
    if let Mode::Insert = app.mode {
        let input_len = app.input.len() as u16;
        f.set_cursor_position((
            chunks[1].x + input_len + 1, chunks[1].y + 1
        ));
    }
}

fn parse_color(s: &str) -> Color {
    if s.starts_with('#') && s.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[1..3], 16),
            u8::from_str_radix(&s[3..5], 16),
            u8::from_str_radix(&s[5..7], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    Color::Reset
}
