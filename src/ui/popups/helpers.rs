use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use similar::{ChangeTag, TextDiff};

use crate::ui::ThemeContext;

/// Create a themed block with title and accent border
pub(crate) fn themed_block<'a>(title: &str, theme: &ThemeContext) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(theme.accent))
}

/// Create a themed block with custom border color
pub(crate) fn themed_block_with_color<'a>(title: &str, border_color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(border_color))
}

/// Render a faint shadow behind a popup for visual depth
pub(crate) fn render_popup_shadow(f: &mut Frame, area: Rect, theme: &ThemeContext) {
    if area.width > 2 && area.height > 1 {
        let shadow = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(1),
        };
        f.render_widget(
            Block::default().style(Style::default().bg(theme.shadow())),
            shadow,
        );
    }
}

/// Status bar shared styling helper
pub(crate) fn status_bar_style(theme: &ThemeContext) -> Style {
    Style::default().bg(theme.accent).fg(Color::Black)
}

/// Compute candidate height for the diff view based on available area,
/// clamping to the maximum number of diff lines available.
pub(crate) fn diff_view_height(total_lines: usize, area_height: usize) -> usize {
    if total_lines == 0 {
        return area_height.saturating_sub(2);
    }
    area_height.saturating_sub(2).min(total_lines).max(1)
}

/// Build side-by-side diff paragraphs from original and proposed text.
/// Each side shows lines with diff highlighting (red bg for deletions, green
/// bg for insertions) and respects the shared scroll offset.
pub(crate) fn side_by_side_diff<'a>(
    original: &'a str,
    proposed: &'a str,
    scroll: usize,
    area_height: usize,
    theme: &ThemeContext,
) -> (Paragraph<'a>, Paragraph<'a>) {
    let diff = TextDiff::from_lines(original, proposed);
    let mut left_lines: Vec<Line<'a>> = Vec::new();
    let mut right_lines: Vec<Line<'a>> = Vec::new();

    for change in diff.iter_all_changes() {
        let value = change.value().trim_end_matches('\n');
        let empty = Line::from(vec![]);
        match change.tag() {
            ChangeTag::Equal => {
                left_lines.push(Line::from(Span::raw(value)));
                right_lines.push(Line::from(Span::raw(value)));
            }
            ChangeTag::Delete => {
                left_lines.push(Line::from(Span::styled(
                    value,
                    Style::default()
                        .bg(theme.diff_delete_bg())
                        .fg(theme.diff_delete_fg()),
                )));
                right_lines.push(empty);
            }
            ChangeTag::Insert => {
                left_lines.push(empty);
                right_lines.push(Line::from(Span::styled(
                    value,
                    Style::default()
                        .bg(theme.diff_insert_bg())
                        .fg(theme.diff_insert_fg()),
                )));
            }
        }
    }

    let height = diff_view_height(left_lines.len().max(right_lines.len()), area_height);
    let (scroll_adj, _extra) = if scroll > height.saturating_sub(1) {
        (height.saturating_sub(1), 0)
    } else {
        (scroll, 0)
    };

    let left = Paragraph::new(left_lines)
        .block(themed_block_with_color("Original", theme.error()))
        .scroll((scroll_adj as u16, 0));
    let right = Paragraph::new(right_lines)
        .block(themed_block_with_color("Proposed", theme.success()))
        .scroll((scroll_adj as u16, 0));

    (left, right)
}

/// Build a unified-diff paragraph from original and proposed text.
/// Lines are prefixed with `+`/`-`/` ` and coloured accordingly.
pub(crate) fn unified_diff<'a>(
    original: &'a str,
    proposed: &'a str,
    scroll: usize,
    area_height: usize,
    theme: &ThemeContext,
) -> Paragraph<'a> {
    let diff = TextDiff::from_lines(original, proposed);
    let mut lines: Vec<Line<'a>> = Vec::new();

    for change in diff.iter_all_changes() {
        let value = change.value().trim_end_matches('\n');
        match change.tag() {
            ChangeTag::Equal => {
                lines.push(Line::from(Span::raw(format!(" {}", value))));
            }
            ChangeTag::Delete => {
                lines.push(Line::from(Span::styled(
                    format!("-{}", value),
                    Style::default()
                        .fg(theme.error())
                        .add_modifier(Modifier::BOLD),
                )));
            }
            ChangeTag::Insert => {
                lines.push(Line::from(Span::styled(
                    format!("+{}", value),
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD),
                )));
            }
        }
    }

    let height = diff_view_height(lines.len(), area_height);
    let scroll_adj = scroll.min(height.saturating_sub(1));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Unified Diff ")
        .border_style(Style::default().fg(theme.accent));

    Paragraph::new(lines)
        .block(block)
        .scroll((scroll_adj as u16, 0))
}
