use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::helpers::*;
use crate::app::{App, ChangeStatus};
use crate::ui::ThemeContext;
use crate::ui::layout::centered_rect;

pub fn draw_review_popup(f: &mut Frame, app: &App, theme: &ThemeContext) {
    if app.proposed_changes.is_empty() {
        return;
    }

    let area = centered_rect(90, 90, f.area());
    render_popup_shadow(f, area, theme);
    f.render_widget(ratatui::widgets::Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // Main content
                Constraint::Length(3), // Status bar
            ]
            .as_ref(),
        )
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30), // File list
                Constraint::Percentage(70), // Diff view
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    // File list (left panel)
    let file_list: Vec<ListItem> = app
        .proposed_changes
        .iter()
        .enumerate()
        .map(|(i, change)| {
            let status_icon = match change.status {
                ChangeStatus::Pending => "○",
                ChangeStatus::Approved => "✓",
                ChangeStatus::Denied => "✗",
            };
            let style = match change.status {
                ChangeStatus::Pending => Style::default().fg(theme.warning()),
                ChangeStatus::Approved => Style::default().fg(theme.success()),
                ChangeStatus::Denied => Style::default().fg(theme.error()),
            };
            let prefix = if i == app.plan_review_index {
                "> "
            } else {
                "  "
            };
            ListItem::new(Line::from(vec![Span::styled(
                format!("{}{} {}", prefix, status_icon, change.path),
                style,
            )]))
        })
        .collect();

    let file_list_widget = List::new(file_list)
        .block(themed_block("Files", theme))
        .highlight_style(Style::default().bg(theme.dim()));
    f.render_widget(file_list_widget, main_chunks[0]);

    // Diff view (right panel)
    let area_height = main_chunks[1].height as usize;
    let scroll = app.plan_review_scroll;

    if let Some(change) = app.proposed_changes.get(app.plan_review_index) {
        if app.review_show_unified {
            let unified = unified_diff(
                &change.original,
                &change.proposed,
                scroll,
                area_height,
                theme,
            );
            f.render_widget(unified, main_chunks[1]);
        } else {
            let (original, proposed) = side_by_side_diff(
                &change.original,
                &change.proposed,
                scroll,
                area_height,
                theme,
            );
            let diff_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(main_chunks[1]);
            f.render_widget(original, diff_chunks[0]);
            f.render_widget(proposed, diff_chunks[1]);
        }
    }

    // Status bar
    let approved_count = app
        .proposed_changes
        .iter()
        .filter(|c| c.status == ChangeStatus::Approved)
        .count();
    let denied_count = app
        .proposed_changes
        .iter()
        .filter(|c| c.status == ChangeStatus::Denied)
        .count();
    let pending_count = app
        .proposed_changes
        .iter()
        .filter(|c| c.status == ChangeStatus::Pending)
        .count();

    let view_label = if app.review_show_unified {
        "Unified"
    } else {
        "Side-by-Side"
    };

    let status_text = format!(
        " [↑/↓] Navigate | [j/k] Scroll | [u] {} | [A]pprove [D]eny | [Shift+A] All | [Enter] Exec | [Esc] Cancel | {}P {}A {}D ",
        view_label, pending_count, approved_count, denied_count
    );
    let status = Paragraph::new(status_text)
        .style(status_bar_style(theme))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[1]);
}
