use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
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
            Mode::Review => Style::default().fg(fg_color),
        })
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Input ")
            .border_style(match app.mode {
                Mode::Normal => Style::default().fg(border_color),
                Mode::Insert => Style::default().fg(accent_color),
                Mode::Review => Style::default().fg(border_color),
            }));
    f.render_widget(input, chunks[1]);
    
    // Status
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Review => "REVIEW",
    };
    
    let stats = app.llm_client.usage_stats.try_lock();
    let stats_str = if let Ok(s) = stats {
        format!(" | In: {} | Out: {} | Cost: ${:.4}", s.input_tokens, s.output_tokens, s.total_cost)
    } else {
        String::new()
    };

    let status = Paragraph::new(format!("-- {} --{}", mode_str, stats_str))
        .style(Style::default().bg(accent_color).fg(fg_color));
    f.render_widget(status, chunks[2]);
    
    // Cursor handling
    if let Mode::Insert = app.mode {
        let input_len = app.input.len() as u16;
        f.set_cursor_position((
            chunks[1].x + input_len + 1, chunks[1].y + 1
        ));
    }

    if let Mode::Review = app.mode {
        if let Some(change) = app.proposed_changes.last() {
            let area = centered_rect(80, 80, f.area());
            f.render_widget(Clear, area);
            
            let diff_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);

            let original = Paragraph::new(change.original.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Original ")
                .border_style(Style::default().fg(Color::Red)));
            let proposed = Paragraph::new(change.proposed.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Proposed ")
                .border_style(Style::default().fg(Color::Green)));

            f.render_widget(original, diff_chunks[0]);
            f.render_widget(proposed, diff_chunks[1]);
            
            let hint = Paragraph::new(" [A]pprove | [D]eny ")
                .style(Style::default().bg(accent_color).fg(fg_color))
                .block(Block::default().borders(Borders::BOTTOM));
            // Render hint at the bottom of the popup
            let hint_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
            f.render_widget(hint, hint_area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
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
