use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use super::helpers::*;
use crate::app::App;
use crate::ui::ThemeContext;
use crate::ui::layout::centered_rect;

pub fn draw_servers_popup(f: &mut Frame, app: &App, theme: &ThemeContext) {
    let area = centered_rect(90, 80, f.area());
    render_popup_shadow(f, area, theme);
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
        Style::default().fg(theme.ghost())
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
                Style::default().fg(theme.success())
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
                Span::styled("Name: ", Style::default().fg(theme.warning())),
                Span::styled(
                    name,
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Description: ",
                Style::default().fg(theme.warning()),
            )]),
            Line::from(desc.as_str()),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Command: ",
                Style::default().fg(theme.warning()),
            )]),
            Line::from(cmd_str),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.warning())),
                Span::styled(
                    if is_installed {
                        "Installed"
                    } else {
                        "Not installed"
                    },
                    Style::default().fg(if is_installed {
                        theme.success()
                    } else {
                        theme.error()
                    }),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Note: ", Style::default().fg(theme.warning())),
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

pub fn draw_skill_browser(f: &mut Frame, app: &App, theme: &ThemeContext) {
    let area = centered_rect(70, 60, f.area());
    render_popup_shadow(f, area, theme);
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
                Style::default().fg(theme.success())
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
        let status_color = if *active {
            theme.success()
        } else {
            theme.error()
        };

        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.warning())),
                Span::styled(
                    name,
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Description: ",
                Style::default().fg(theme.warning()),
            )]),
            Line::from(desc.as_str()),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.warning())),
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
                Span::styled("Note: ", Style::default().fg(theme.warning())),
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
    render_popup_shadow(f, area, theme);
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
                Style::default().fg(theme.success())
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
        let status_color = if *enabled {
            theme.success()
        } else {
            theme.error()
        };

        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.warning())),
                Span::styled(
                    name,
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Description: ",
                Style::default().fg(theme.warning()),
            )]),
            Line::from(desc.as_str()),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.warning())),
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
                Span::styled("Note: ", Style::default().fg(theme.warning())),
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
