use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

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
            Block::default().style(Style::default().bg(Color::Rgb(10, 10, 10))),
            shadow,
        );
    }
}

/// Status bar shared styling helper
fn status_bar_style(theme: &ThemeContext) -> Style {
    Style::default().bg(theme.accent).fg(Color::Black)
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
    if let Some(change) = app.proposed_changes.get(app.plan_review_index) {
        let diff_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunks[1]);

        let original = Paragraph::new(change.original.as_str())
            .block(themed_block_with_color("Original", Color::Red));
        let proposed = Paragraph::new(change.proposed.as_str())
            .block(themed_block_with_color("Proposed", Color::Green));

        f.render_widget(original, diff_chunks[0]);
        f.render_widget(proposed, diff_chunks[1]);
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

    let status_text = format!(
        " [↑/↓] Navigate | [A]pprove [D]eny | [Shift+A] Approve All | [Enter] Execute Approved | [Esc] Cancel | Pending: {} Approved: {} Denied: {} ",
        pending_count, approved_count, denied_count
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

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[1]);

    // Left panel: Available MCP Servers list
    let available_servers: Vec<ListItem> = app
        .mcp_browser_items
        .iter()
        .enumerate()
        .map(|(i, (name, _desc, _))| {
            let is_installed = app.config.mcp.contains_key(name);
            let prefix = if i == app.mcp_browser_selected {
                "> "
            } else {
                "  "
            };
            let suffix = if is_installed { " [INSTALLED]" } else { "" };
            let style = if i == app.mcp_browser_selected {
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
    f.render_widget(status, chunks[2]);
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
        Line::from("  Ctrl+M          MCP server showcase"),
        Line::from("  Ctrl+G          Mission Control (task DAG)"),
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
