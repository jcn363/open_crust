use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::ui::ThemeContext;

/// Render the @ file picker popup overlay.
pub fn draw_file_picker(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    // Position picker near the input area (bottom of screen)
    let picker_height = 12.min(app.file_picker_results.len() as u16 + 3);
    let picker_width = 60.min(area.width.saturating_sub(4));

    let picker_area = Rect {
        x: area.x + 1,
        y: area.height.saturating_sub(picker_height + 4),
        width: picker_width,
        height: picker_height,
    };

    f.render_widget(Clear, picker_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(picker_area);

    // Query bar
    let query_text = format!("@{}", app.file_picker_query);
    let query_para = Paragraph::new(query_text)
        .style(Style::default().fg(theme.accent))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" File Search ")
                .border_style(Style::default().fg(theme.accent)),
        );
    f.render_widget(query_para, chunks[0]);

    // Results list
    let max_visible = chunks[1].height.saturating_sub(2) as usize;
    let results: Vec<ListItem> = app
        .file_picker_results
        .iter()
        .enumerate()
        .skip(app.file_picker_scroll)
        .take(max_visible)
        .map(|(idx, path)| {
            let is_selected = idx == app.file_picker_selected;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            let icon = if is_selected { "▸ " } else { "  " };
            ListItem::new(Line::from(Span::styled(format!("{}{}", icon, path), style)))
        })
        .collect();

    let results_list = List::new(results).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} files ", app.file_picker_results.len()))
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(results_list, chunks[1]);
}
