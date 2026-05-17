//! Popup rendering for the TUI
//!
//! Draws modal overlays: review confirmation, MCP server browser, command
//! palette, skill browser, and help popup. Each popup is a self-contained
//! rendering function with keyboard navigation.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use similar::{ChangeTag, TextDiff};

use super::ThemeContext;
use super::layout::centered_rect;
use crate::app::{App, ChangeStatus};

/// Create a themed block with title and accent border
fn themed_block<'a>(title: &str, theme: &ThemeContext) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(theme.accent))
}

/// Create a themed block with custom border color
fn themed_block_with_color<'a>(title: &str, border_color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(border_color))
}

/// Render a faint shadow behind a popup for visual depth
fn render_popup_shadow(f: &mut Frame, area: ratatui::layout::Rect) {
    if area.width > 2 && area.height > 1 {
        let shadow = ratatui::layout::Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(1),
        };
        f.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(10, 9, 9))),
            shadow,
        );
    }
}

/// Status bar shared styling helper
fn status_bar_style(theme: &ThemeContext) -> Style {
    Style::default().bg(theme.accent).fg(Color::Black)
}

/// Compute candidate height for the diff view based on available area,
/// clamping to the maximum number of diff lines available.
fn diff_view_height(total_lines: usize, area_height: usize) -> usize {
    if total_lines == 0 {
        return area_height.saturating_sub(2);
    }
    area_height.saturating_sub(2).min(total_lines).max(1)
}

/// Build side-by-side diff paragraphs from original and proposed text.
/// Each side shows lines with diff highlighting (red bg for deletions, green
/// bg for insertions) and respects the shared scroll offset.
fn side_by_side_diff<'a>(
    original: &'a str,
    proposed: &'a str,
    scroll: usize,
    area_height: usize,
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
                        .bg(Color::Rgb(60, 20, 20))
                        .fg(Color::Rgb(200, 130, 130)),
                )));
                right_lines.push(empty);
            }
            ChangeTag::Insert => {
                left_lines.push(empty);
                right_lines.push(Line::from(Span::styled(
                    value,
                    Style::default()
                        .bg(Color::Rgb(20, 50, 20))
                        .fg(Color::Rgb(130, 200, 130)),
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
        .block(themed_block_with_color("Original", Color::Red))
        .scroll((scroll_adj as u16, 0));
    let right = Paragraph::new(right_lines)
        .block(themed_block_with_color("Proposed", Color::Green))
        .scroll((scroll_adj as u16, 0));

    (left, right)
}

/// Build a unified-diff paragraph from original and proposed text.
/// Lines are prefixed with `+`/`-`/` ` and coloured accordingly.
fn unified_diff<'a>(
    original: &'a str,
    proposed: &'a str,
    scroll: usize,
    area_height: usize,
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
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
            }
            ChangeTag::Insert => {
                lines.push(Line::from(Span::styled(
                    format!("+{}", value),
                    Style::default()
                        .fg(Color::Green)
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
        .border_style(Style::default().fg(Color::Cyan));

    Paragraph::new(lines)
        .block(block)
        .scroll((scroll_adj as u16, 0))
}

pub fn draw_review_popup(f: &mut Frame, app: &App, theme: &ThemeContext) {
    if app.proposed_changes.is_empty() {
        return;
    }

    let area = centered_rect(90, 90, f.area());
    render_popup_shadow(f, area);
    f.render_widget(Clear, area);

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
                ChangeStatus::Pending => Style::default().fg(Color::Yellow),
                ChangeStatus::Approved => Style::default().fg(Color::Green),
                ChangeStatus::Denied => Style::default().fg(Color::Red),
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
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(file_list_widget, main_chunks[0]);

    // Diff view (right panel)
    let area_height = main_chunks[1].height as usize;
    let scroll = app.plan_review_scroll;

    if let Some(change) = app.proposed_changes.get(app.plan_review_index) {
        if app.review_show_unified {
            let unified = unified_diff(&change.original, &change.proposed, scroll, area_height);
            f.render_widget(unified, main_chunks[1]);
        } else {
            let (original, proposed) =
                side_by_side_diff(&change.original, &change.proposed, scroll, area_height);
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

pub fn draw_servers_popup(f: &mut Frame, app: &App, theme: &ThemeContext) {
    let area = centered_rect(90, 80, f.area());
    render_popup_shadow(f, area);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(3), // Filter bar
                Constraint::Min(1),    // Content
                Constraint::Length(3), // Status bar
            ]
            .as_ref(),
        )
        .split(area);

    // Title
    let title = Paragraph::new("MCP Server Browser")
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .block(themed_block("MCP Server Browser", theme));
    f.render_widget(title, chunks[0]);

    // Filter bar
    let filter_text = if app.mcp_input.is_empty() {
        "Type to filter servers..."
    } else {
        &app.mcp_input
    };
    let filter_style = if app.mcp_input.is_empty() {
        Style::default().fg(Color::Rgb(73, 72, 71))
    } else {
        Style::default().fg(theme.fg)
    };
    let filter_para = Paragraph::new(filter_text).style(filter_style).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Filter ")
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(filter_para, chunks[1]);

    // Filter server list
    let filter_lower = app.mcp_input.to_lowercase();
    let filtered_items: Vec<_> = app
        .mcp_browser_items
        .iter()
        .enumerate()
        .filter(|(_, (name, desc, _))| {
            filter_lower.is_empty()
                || name.to_lowercase().contains(&filter_lower)
                || desc.to_lowercase().contains(&filter_lower)
        })
        .collect();

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[2]);

    // Left panel: Available MCP Servers list
    let available_servers: Vec<ListItem> = filtered_items
        .iter()
        .map(|(orig_idx, (name, _desc, _))| {
            let is_installed = app.config.mcp.contains_key(name.as_str());
            let prefix = if *orig_idx == app.mcp_browser_selected {
                "> "
            } else {
                "  "
            };
            let suffix = if is_installed { " [INSTALLED]" } else { "" };
            let style = if *orig_idx == app.mcp_browser_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if is_installed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(Line::from(vec![Span::styled(
                format!("{}{}{}", prefix, name, suffix),
                style,
            )]))
        })
        .collect();

    let list = List::new(available_servers).block(themed_block("Available Servers", theme));
    f.render_widget(list, content_chunks[0]);

    // Right panel: Selected server details
    if let Some((name, desc, cmd)) = app.mcp_browser_items.get(app.mcp_browser_selected) {
        let is_installed = app.config.mcp.contains_key(name);
        let cmd_str = cmd.join(" ");

        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    name,
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Description: ",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(desc.as_str()),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Command: ",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(cmd_str),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if is_installed {
                        "Installed"
                    } else {
                        "Not installed"
                    },
                    Style::default().fg(if is_installed {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Note: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if is_installed {
                        "Restart opencrust to use this server."
                    } else {
                        "Press [Enter] to install."
                    },
                    Style::default().fg(theme.fg),
                ),
            ]),
        ];

        let details_para = Paragraph::new(details)
            .block(themed_block("Server Details", theme))
            .wrap(Wrap { trim: true });
        f.render_widget(details_para, content_chunks[1]);
    }

    // Unified status bar
    let status_text =
        if let Some((name, _, _)) = app.mcp_browser_items.get(app.mcp_browser_selected) {
            if app.config.mcp.contains_key(name) {
                " [↑/↓] Navigate | [Esc] Close "
            } else {
                " [↑/↓] Navigate | [Enter] Install | [Esc] Close "
            }
        } else {
            " [↑/↓] Navigate | [Esc] Close "
        };
    let status = Paragraph::new(status_text)
        .style(status_bar_style(theme))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[3]);
}

pub fn draw_command_palette(f: &mut Frame, app: &App, theme: &ThemeContext) {
    let area = centered_rect(60, 35, f.area());
    render_popup_shadow(f, area);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(1),    // Items
                Constraint::Length(3), // Status
            ]
            .as_ref(),
        )
        .split(area);

    // Title
    let title = Paragraph::new("Command Palette")
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .block(themed_block("Command Palette", theme));
    f.render_widget(title, chunks[0]);

    // Items
    let items = [
        (
            "Switch Provider",
            format!("Current: {:?}", app.config.provider),
        ),
        ("Switch Model", format!("Current: {}", app.config.model)),
        ("Clear Context", "Clear conversation history".to_string()),
        ("MCP Browser", "Manage MCP servers".to_string()),
    ];

    let menu_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (label, detail))| {
            let prefix = if i == app.command_palette_selected {
                "> "
            } else {
                "  "
            };
            let style = if i == app.command_palette_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix.to_string(), style),
                Span::styled(label.to_string(), style),
                Span::styled(format!("  {}", detail), style.dim()),
            ]))
        })
        .collect();

    let list = List::new(menu_items).block(themed_block("Commands", theme));
    f.render_widget(list, chunks[1]);

    // Status
    let status = Paragraph::new("[↑/↓] Navigate | [Enter] Select | [Esc] Cancel")
        .style(status_bar_style(theme))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

pub fn draw_skill_browser(f: &mut Frame, app: &App, theme: &ThemeContext) {
    let area = centered_rect(70, 60, f.area());
    render_popup_shadow(f, area);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(1),    // Content
                Constraint::Length(3), // Status
            ]
            .as_ref(),
        )
        .split(area);

    // Title
    let title = Paragraph::new("Skill Browser (Ctrl+Shift+K)")
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .block(themed_block("Skill Browser (Ctrl+Shift+K)", theme));
    f.render_widget(title, chunks[0]);

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[1]);

    // Left panel: Available Skills list
    let skill_list: Vec<ListItem> = app
        .skill_browser_items
        .iter()
        .enumerate()
        .map(|(i, (name, _desc, active))| {
            let prefix = if i == app.skill_browser_selected {
                "> "
            } else {
                "  "
            };
            let status = if *active { "[ACTIVE]" } else { "[INACTIVE]" };
            let style = if i == app.skill_browser_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if *active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(Line::from(vec![Span::styled(
                format!("{}{} {}", prefix, name, status),
                style,
            )]))
        })
        .collect();

    let list = List::new(skill_list).block(themed_block("Available Skills", theme));
    f.render_widget(list, content_chunks[0]);

    // Right panel: Selected skill details
    if let Some((name, desc, active)) = app.skill_browser_items.get(app.skill_browser_selected) {
        let status_text = if *active { "ACTIVE" } else { "INACTIVE" };
        let status_color = if *active { Color::Green } else { Color::Red };

        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    name,
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Description: ",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(desc.as_str()),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("Note: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if *active {
                        "Skill is active and will be used by the LLM. Press [Enter] to deactivate."
                    } else {
                        "Skill is inactive. Press [Enter] to activate."
                    },
                    Style::default().fg(theme.fg),
                ),
            ]),
        ];

        let details_para = Paragraph::new(details)
            .block(themed_block("Skill Details", theme))
            .wrap(Wrap { trim: true });
        f.render_widget(details_para, content_chunks[1]);
    }

    // Unified status bar
    let status_text = "[↑/↓] Navigate | [Enter] Toggle Active | [Esc/q] Close";
    let status = Paragraph::new(status_text)
        .style(status_bar_style(theme))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

pub fn draw_plugin_browser(f: &mut Frame, app: &App, theme: &ThemeContext) {
    let area = centered_rect(70, 60, f.area());
    render_popup_shadow(f, area);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(1),    // Content
                Constraint::Length(3), // Status
            ]
            .as_ref(),
        )
        .split(area);

    // Title
    f.render_widget(
        Paragraph::new("Plugin Browser (Ctrl+P)")
            .style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .block(themed_block("Plugin Browser (Ctrl+P)", theme)),
        chunks[0],
    );

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[1]);

    // Left panel: Installed Plugins list
    let plugin_list: Vec<ListItem> = app
        .plugin_browser_items
        .iter()
        .enumerate()
        .map(|(i, (name, _desc, enabled))| {
            let prefix = if i == app.plugin_browser_selected {
                "> "
            } else {
                "  "
            };
            let status = if *enabled { "[ENABLED]" } else { "[DISABLED]" };
            let style = if i == app.plugin_browser_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if *enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(Line::from(vec![Span::styled(
                format!("{}{} {}", prefix, name, status),
                style,
            )]))
        })
        .collect();

    let list = List::new(plugin_list).block(themed_block("Installed Plugins", theme));
    f.render_widget(list, content_chunks[0]);

    // Right panel: Selected plugin details
    if let Some((name, desc, enabled)) = app.plugin_browser_items.get(app.plugin_browser_selected) {
        let status_text = if *enabled { "ENABLED" } else { "DISABLED" };
        let status_color = if *enabled { Color::Green } else { Color::Red };

        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    name,
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Description: ",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(desc.as_str()),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("Note: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if *enabled {
                        "Plugin is enabled. Press [Enter] to disable."
                    } else {
                        "Plugin is disabled. Press [Enter] to enable."
                    },
                    Style::default().fg(theme.fg),
                ),
            ]),
        ];

        let details_para = Paragraph::new(details)
            .block(themed_block("Plugin Details", theme))
            .wrap(Wrap { trim: true });
        f.render_widget(details_para, content_chunks[1]);
    }

    // Unified status bar
    let status_text = "[↑/↓] Navigate | [Enter] Toggle Enable | [Esc/q] Close";
    let status = Paragraph::new(status_text)
        .style(status_bar_style(theme))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

pub fn draw_help_popup(f: &mut Frame, _app: &App, theme: &ThemeContext) {
    let area = centered_rect(65, 65, f.area());
    render_popup_shadow(f, area);
    f.render_widget(Clear, area);

    let help_lines = vec![
        Line::from(vec![Span::styled(
            " OpenCrust Keyboard Help ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── Movement ──",
            Style::default()
                .fg(theme.border)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Tab             Switch between Chat/Tasks tabs"),
        Line::from("  ↑/↓             Scroll message list"),
        Line::from("  PgUp/PgDn       Scroll by 10 lines"),
        Line::from("  Home/End        Jump to top/bottom"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── Modes ──",
            Style::default()
                .fg(theme.border)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  i               Enter Insert mode (type)"),
        Line::from("  Esc             Return to Normal mode"),
        Line::from("  ?               Toggle this help screen"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── Actions ──",
            Style::default()
                .fg(theme.border)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter           Send message (in Insert mode)"),
        Line::from("  Ctrl+B          Toggle file sidebar"),
        Line::from("  Ctrl+K          Command palette"),
        Line::from("  Ctrl+Shift+K    Skill browser"),
        Line::from("  Ctrl+Shift+P    Plugin browser"),
        Line::from("  Ctrl+P          Toggle plan mode"),
        Line::from("  Ctrl+M          MCP server showcase"),
        Line::from("  Ctrl+G          Mission Control (task DAG)"),
        Line::from("  Ctrl+T          Spawn background task"),
        Line::from("  Ctrl+F          Format selected file"),
        Line::from("  Alt+V           Toggle Vim mode (insert)"),
        Line::from("  Ctrl+Q          Quit OpenCrust"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── Vim Mode (Insert) ──",
            Style::default()
                .fg(theme.border)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  h/l             Move cursor left/right"),
        Line::from("  w/b             Next/previous word"),
        Line::from("  0/$             Line start/end"),
        Line::from("  d/c             Delete line"),
        Line::from("  y               Yank (copy) input"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── Commands ──",
            Style::default()
                .fg(theme.border)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  /init           Initialize project rules"),
        Line::from("  /provider <n>   Switch LLM provider"),
        Line::from("  /model <name>   Switch model"),
        Line::from("  /goal <desc>    Set autonomous goal"),
        Line::from("  /goal-clear     Clear active goal"),
        Line::from("  /undo /redo     Git undo/redo"),
        Line::from("  /share          Share conversation to JSON"),
        Line::from("  /format         Format selected sidebar file"),
        Line::from("  /format <path>  Format specific file"),
        Line::from("  @               Open file fuzzy search picker"),
    ];

    let help_para = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(theme.accent)),
        )
        .style(Style::default().fg(theme.fg));
    f.render_widget(help_para, area);
}

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
