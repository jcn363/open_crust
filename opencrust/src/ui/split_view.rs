//! Split view rendering for inline diffs and side-by-side comparison
//!
//! Provides three modes: SideBySide, InlineUnified, InlineSplit

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, SplitViewMode};
use crate::ui::ThemeContext;

/// Draw the split view overlay covering the full terminal area.
pub fn draw_split_view(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    match app.split_view_mode {
        SplitViewMode::SideBySide => draw_side_by_side(f, app, area, theme),
        SplitViewMode::InlineUnified => draw_inline_unified(f, app, area, theme),
        SplitViewMode::InlineSplit => draw_inline_split(f, app, area, theme),
    }
}

/// Draw side-by-side diff panes.
fn draw_side_by_side(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Length(1), // gutter
            ratatui::layout::Constraint::Percentage(50),
        ])
        .split(area);

    // Left pane: Original
    let left_lines = diff_lines_for_pane(
        app.split_left_content.as_deref().unwrap_or_default(),
        app.split_right_content.as_deref().unwrap_or_default(),
        true,
    );
    let left_visible = scroll_lines(
        &left_lines,
        app.split_left_scroll,
        chunks[0].height.saturating_sub(2) as usize,
    );
    let left_para = Paragraph::new(left_visible)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Original ")
                .border_style(Style::default().fg(theme.accent)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(left_para, chunks[0]);

    // Gutter
    let gutter_style = Style::default().fg(theme.dim());
    let gutter_spans: Vec<Line> = vec![Line::from(Span::styled("│", gutter_style))];
    let gutter_para = Paragraph::new(gutter_spans);
    f.render_widget(gutter_para, chunks[1]);

    // Right pane: Proposed
    let right_lines = diff_lines_for_pane(
        app.split_left_content.as_deref().unwrap_or_default(),
        app.split_right_content.as_deref().unwrap_or_default(),
        false,
    );
    let right_visible = scroll_lines(
        &right_lines,
        app.split_right_scroll,
        chunks[2].height.saturating_sub(2) as usize,
    );
    let right_para = Paragraph::new(right_visible)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Proposed ")
                .border_style(Style::default().fg(theme.accent)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(right_para, chunks[2]);
}

/// Draw inline unified diff.
fn draw_inline_unified(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let left = app.split_left_content.as_deref().unwrap_or("");
    let right = app.split_right_content.as_deref().unwrap_or("");

    let hunks = compute_inline_hunks(left, right);
    let total_lines: usize = hunks.iter().map(|h| h.lines.len()).sum();
    let visible_height = area.height.saturating_sub(2) as usize;

    let mut visible_lines: Vec<Line> = Vec::new();
    let mut skipped = 0;
    for hunk in &hunks {
        for line in &hunk.lines {
            if skipped < app.split_left_scroll {
                skipped += 1;
                continue;
            }
            if visible_lines.len() >= visible_height {
                break;
            }
            visible_lines.push(line.clone());
        }
        if visible_lines.len() >= visible_height {
            break;
        }
    }

    let para = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Unified Diff ({} lines) ", total_lines))
                .border_style(Style::default().fg(theme.accent)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

/// Draw inline split diff (interleaved hunks).
fn draw_inline_split(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let left = app.split_left_content.as_deref().unwrap_or("");
    let right = app.split_right_content.as_deref().unwrap_or("");

    let hunks = compute_inline_hunks(left, right);
    let mut display_lines: Vec<Line> = Vec::new();

    for hunk in &hunks {
        if hunk.is_change {
            for del_line in &hunk.deleted {
                display_lines.push(del_line.clone());
            }
            for ins_line in &hunk.inserted {
                display_lines.push(ins_line.clone());
            }
            display_lines.push(Line::from(Span::styled(
                "  ···",
                Style::default().fg(theme.dim()),
            )));
        } else {
            display_lines.extend(hunk.lines.clone());
        }
    }

    let total_lines = display_lines.len();
    let visible_height = area.height.saturating_sub(2) as usize;
    let visible: Vec<Line> = display_lines
        .into_iter()
        .skip(app.split_left_scroll)
        .take(visible_height)
        .collect();

    let para = Paragraph::new(visible)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Split Diff ({} lines) ", total_lines))
                .border_style(Style::default().fg(theme.accent)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

/// A diff hunk for inline rendering.
struct InlineHunk {
    lines: Vec<Line<'static>>,
    is_change: bool,
    deleted: Vec<Line<'static>>,
    inserted: Vec<Line<'static>>,
}

/// Compute inline hunks from two texts.
fn compute_inline_hunks(left: &str, right: &str) -> Vec<InlineHunk> {
    let diff = similar::TextDiff::from_lines(left, right);
    let mut hunks: Vec<InlineHunk> = Vec::new();

    for op in diff.ops() {
        match op {
            similar::DiffOp::Equal {
                old_index: _,
                new_index,
                len,
            } => {
                let lines: Vec<Line<'static>> = (0..*len)
                    .map(|i| {
                        let text = right.lines().nth(new_index + i).unwrap_or("");
                        Line::from(Span::styled(
                            format!("  {}", text),
                            Style::default().fg(Color::White),
                        ))
                    })
                    .collect();
                hunks.push(InlineHunk {
                    lines,
                    is_change: false,
                    deleted: Vec::new(),
                    inserted: Vec::new(),
                });
            }
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => {
                let deleted: Vec<Line<'static>> = (0..*old_len)
                    .map(|i| {
                        let text = left.lines().nth(old_index + i).unwrap_or("");
                        Line::from(Span::styled(
                            format!("- {}", text),
                            Style::default()
                                .fg(Color::Rgb(200, 130, 130))
                                .bg(Color::Rgb(60, 20, 20)),
                        ))
                    })
                    .collect();
                hunks.push(InlineHunk {
                    lines: deleted.clone(),
                    is_change: true,
                    deleted,
                    inserted: Vec::new(),
                });
            }
            similar::DiffOp::Insert {
                new_index, new_len, ..
            } => {
                let inserted: Vec<Line<'static>> = (0..*new_len)
                    .map(|i| {
                        let text = right.lines().nth(new_index + i).unwrap_or("");
                        Line::from(Span::styled(
                            format!("+ {}", text),
                            Style::default()
                                .fg(Color::Rgb(130, 200, 130))
                                .bg(Color::Rgb(20, 50, 20)),
                        ))
                    })
                    .collect();
                hunks.push(InlineHunk {
                    lines: inserted.clone(),
                    is_change: true,
                    deleted: Vec::new(),
                    inserted,
                });
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let deleted: Vec<Line<'static>> = (0..*old_len)
                    .map(|i| {
                        let text = left.lines().nth(old_index + i).unwrap_or("");
                        Line::from(Span::styled(
                            format!("- {}", text),
                            Style::default()
                                .fg(Color::Rgb(200, 130, 130))
                                .bg(Color::Rgb(60, 20, 20)),
                        ))
                    })
                    .collect();
                let inserted: Vec<Line<'static>> = (0..*new_len)
                    .map(|i| {
                        let text = right.lines().nth(new_index + i).unwrap_or("");
                        Line::from(Span::styled(
                            format!("+ {}", text),
                            Style::default()
                                .fg(Color::Rgb(130, 200, 130))
                                .bg(Color::Rgb(20, 50, 20)),
                        ))
                    })
                    .collect();
                let mut lines = Vec::new();
                lines.extend(deleted.iter().cloned());
                lines.extend(inserted.iter().cloned());
                hunks.push(InlineHunk {
                    lines,
                    is_change: true,
                    deleted,
                    inserted,
                });
            }
        }
    }

    hunks
}

/// Compute diff lines for side-by-side view with gutter prefixes.
fn diff_lines_for_pane<'a>(left: &'a str, right: &'a str, is_left: bool) -> Vec<Line<'static>> {
    let diff = similar::TextDiff::from_lines(left, right);
    let mut lines: Vec<Line<'static>> = Vec::new();

    for op in diff.ops() {
        match op {
            similar::DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                for i in 0..*len {
                    let text = if is_left {
                        left.lines().nth(old_index + i).unwrap_or("")
                    } else {
                        right.lines().nth(new_index + i).unwrap_or("")
                    };
                    lines.push(Line::from(Span::styled(
                        format!("│ {}", text),
                        Style::default().fg(Color::White),
                    )));
                }
            }
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => {
                if is_left {
                    for i in 0..*old_len {
                        let text = left.lines().nth(old_index + i).unwrap_or("");
                        lines.push(Line::from(Span::styled(
                            format!("- {}", text),
                            Style::default()
                                .fg(Color::Rgb(200, 130, 130))
                                .bg(Color::Rgb(60, 20, 20)),
                        )));
                    }
                } else {
                    for _ in 0..*old_len {
                        lines.push(Line::from(Span::styled(
                            "│ ",
                            Style::default().fg(Color::Rgb(50, 50, 50)),
                        )));
                    }
                }
            }
            similar::DiffOp::Insert {
                new_index, new_len, ..
            } => {
                if !is_left {
                    for i in 0..*new_len {
                        let text = right.lines().nth(new_index + i).unwrap_or("");
                        lines.push(Line::from(Span::styled(
                            format!("+ {}", text),
                            Style::default()
                                .fg(Color::Rgb(130, 200, 130))
                                .bg(Color::Rgb(20, 50, 20)),
                        )));
                    }
                } else {
                    for _ in 0..*new_len {
                        lines.push(Line::from(Span::styled(
                            "│ ",
                            Style::default().fg(Color::Rgb(50, 50, 50)),
                        )));
                    }
                }
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let max_len = (*old_len).max(*new_len);
                for i in 0..max_len {
                    if i < *old_len && is_left {
                        let text = left.lines().nth(old_index + i).unwrap_or("");
                        lines.push(Line::from(Span::styled(
                            format!("- {}", text),
                            Style::default()
                                .fg(Color::Rgb(200, 130, 130))
                                .bg(Color::Rgb(60, 20, 20)),
                        )));
                    } else if i < *new_len && !is_left {
                        let text = right.lines().nth(new_index + i).unwrap_or("");
                        lines.push(Line::from(Span::styled(
                            format!("+ {}", text),
                            Style::default()
                                .fg(Color::Rgb(130, 200, 130))
                                .bg(Color::Rgb(20, 50, 20)),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            "│ ",
                            Style::default().fg(Color::Rgb(50, 50, 50)),
                        )));
                    }
                }
            }
        }
    }

    lines
}

/// Apply scroll offset and viewport clipping to a list of lines.
fn scroll_lines(lines: &[Line<'static>], scroll: usize, height: usize) -> Vec<Line<'static>> {
    lines.iter().skip(scroll).take(height).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_view_mode_default_is_side_by_side() {
        assert_eq!(SplitViewMode::default(), SplitViewMode::SideBySide);
    }

    #[test]
    fn diff_lines_for_pane_detects_additions() {
        let left = "line1\nline2";
        let right = "line1\nline2\nline3";
        let lines = diff_lines_for_pane(left, right, false);
        assert!(
            lines
                .iter()
                .any(|l| { l.spans.iter().any(|s| s.content.contains("+ line3")) })
        );
    }

    #[test]
    fn diff_lines_for_pane_detects_deletions() {
        let left = "line1\nline2\nline3";
        let right = "line1\nline3";
        let lines = diff_lines_for_pane(left, right, true);
        assert!(
            lines
                .iter()
                .any(|l| { l.spans.iter().any(|s| s.content.contains("- line2")) })
        );
    }

    #[test]
    fn scroll_lines_clips_correctly() {
        let lines: Vec<Line<'static>> =
            (0..10).map(|i| Line::from(format!("line {}", i))).collect();
        let visible = scroll_lines(&lines, 3, 4);
        assert_eq!(visible.len(), 4);
        assert!(visible[0].to_string().contains("3"));
    }

    #[test]
    fn compute_inline_hunks_on_identical_text() {
        let hunks = compute_inline_hunks("hello\nworld", "hello\nworld");
        assert!(hunks.iter().all(|h| !h.is_change));
    }

    #[test]
    fn compute_inline_hunks_on_different_text() {
        let hunks = compute_inline_hunks("a\nb", "a\nc");
        assert!(hunks.iter().any(|h| h.is_change));
    }
}
