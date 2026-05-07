use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, Wrap, Tabs},
    Frame,
};

use crate::app::{App, Mode, ChangeStatus};

pub fn draw(f: &mut Frame, app: &App) {
    let theme = app.config.theme.as_ref();
    let bg_color = parse_color(theme.map(|t| t.background.as_str()).unwrap_or("#1e1e1e"));
    let fg_color = parse_color(theme.map(|t| t.foreground.as_str()).unwrap_or("#ffffff"));
    let accent_color = parse_color(theme.map(|t| t.accent.as_str()).unwrap_or("#007acc"));
    let border_color = parse_color(theme.map(|t| t.border.as_str()).unwrap_or("#333333"));

    // Background
    let bg_block = Block::default().style(Style::default().bg(bg_color));
    f.render_widget(bg_block, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Input
            Constraint::Length(1), // Status
        ].as_ref())
        .split(f.area());

    // Tabs
    let tab_titles: Vec<Line> = app.tabs.iter()
        .map(|t| Line::from(t.name.as_str()))
        .collect();
    let tabs_widget = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" Views ")
            .border_style(Style::default().fg(border_color)))
        .select(app.active_tab)
        .highlight_style(Style::default().fg(accent_color).add_modifier(Modifier::BOLD));
    f.render_widget(tabs_widget, chunks[0]);

    let main_area = if app.show_sidebar {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25),
                Constraint::Min(0),
            ].as_ref())
            .split(chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(0),
                Constraint::Min(0),
            ].as_ref())
            .split(chunks[1])
    };

    // Sidebar
    if app.show_sidebar {
        let sidebar_items: Vec<ListItem> = app.sidebar_items.iter()
            .map(|i| ListItem::new(Line::from(Span::raw(format!(" {} ", i)))))
            .collect();
        let sidebar_list = List::new(sidebar_items)
            .block(Block::default().borders(Borders::ALL).title(" Files ")
            .border_style(Style::default().fg(border_color)));
        f.render_widget(sidebar_list, main_area[0]);
    }

    // Active tab messages + background tasks
    let mut all_items: Vec<ListItem> = Vec::new();
    
    // Add background tasks if on Tasks tab
    if app.active_tab == 1 {
        let task_items: Vec<ListItem> = app.background_tasks.iter()
            .map(|task| {
                let status_icon = match task.status {
                    crate::app::TaskStatus::Running => "⏳",
                    crate::app::TaskStatus::Completed => "✅",
                    crate::app::TaskStatus::Failed => "❌",
                };
                let time_str = task.started_at.format("%H:%M:%S").to_string();
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", status_icon), Style::default().fg(Color::Yellow)),
                    Span::raw(format!("[{}] {}: {}", task.id, time_str, task.prompt)),
                ]))
            })
            .collect();
        all_items.extend(task_items);
        
        if app.background_tasks.is_empty() {
            all_items.push(ListItem::new(Line::from(Span::raw("No background tasks yet."))));
        }
        
        // Add separator if there are chat messages
        if let Some(tab) = app.tabs.get(app.active_tab) {
            if !tab.messages.is_empty() {
                let sep_line = Line::from(vec![
                    Span::styled("--- Chat Messages ---", Style::default().fg(Color::DarkGray))
                ]);
                all_items.push(ListItem::new(sep_line));
                all_items.extend(tab.messages.iter().map(|m| ListItem::new(Line::from(Span::raw(m)))));
            }
        }
    } else {
        // Chat tab: just show messages
        if let Some(tab) = app.tabs.get(app.active_tab) {
            all_items.extend(tab.messages.iter().map(|m| ListItem::new(Line::from(Span::raw(m)))));
        }
    }
    
    let messages_list = List::new(all_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", app.tabs.get(app.active_tab).map(|t| t.name.as_str()).unwrap_or("Chat")))
            .border_style(Style::default().fg(border_color))
            .style(Style::default().fg(fg_color)));
    f.render_widget(messages_list, main_area[1]);

    // Input Box
    let input = Paragraph::new(app.input.as_str())
     .style(match app.mode {
             Mode::Normal => Style::default().fg(fg_color),
             Mode::Insert => Style::default().fg(accent_color),
             Mode::Review => Style::default().fg(fg_color),
             Mode::Servers => Style::default().fg(fg_color),
             Mode::CommandPalette => Style::default().fg(fg_color),
         })
         .block(Block::default()
             .borders(Borders::ALL)
             .title(" Input ")
             .border_style(match app.mode {
                 Mode::Normal => Style::default().fg(border_color),
                 Mode::Insert => Style::default().fg(accent_color),
                 Mode::Review => Style::default().fg(border_color),
                 Mode::Servers => Style::default().fg(border_color),
                 Mode::CommandPalette => Style::default().fg(border_color),
             }));
    f.render_widget(input, chunks[2]);

    // Status bar
     let mode_str = match app.mode {
         Mode::Normal  => "NORMAL",
         Mode::Insert  => "INSERT",
         Mode::Review  => "REVIEW",
         Mode::Servers => "SERVERS",
         Mode::CommandPalette => "PALETTE",
     };

    let stats = app.llm_client.usage_stats.try_lock();
    let context_budget = app.config.context_limit();
    
    let stats_str = if let Ok(s) = stats {
        let total_tokens = s.total_tokens();
        let context_percent = if context_budget > 0 {
            (total_tokens as f64 / context_budget as f64 * 100.0) as u16
        } else {
            0
        };
        
        format!(
            " | 🤖 {} | Context: {}/{} ({}%) | Cost: ${:.4}",
            app.config.model, total_tokens, context_budget, context_percent, s.total_cost
        )
    } else {
        format!(" | 🤖 {} ", app.config.model)
    };

    let status = Paragraph::new(format!("-- {} -- | Ctrl+B: Sidebar | Tab: Switch view{}", mode_str, stats_str))
        .style(Style::default().fg(accent_color));
    f.render_widget(status, chunks[3]);

    // Cursor handling
    if let Mode::Insert = app.mode {
        let input_len = app.input.len() as u16;
        f.set_cursor_position((
            chunks[2].x + input_len + 1, chunks[2].y + 1
        ));
    }

    if let Mode::Review = app.mode {
        draw_review_popup(f, app);
    } else if let Mode::Servers = app.mode {
        draw_servers_popup(f, app);
    } else if let Mode::CommandPalette = app.mode {
        draw_command_palette(f, app);
    }
}

fn draw_review_popup(f: &mut Frame, app: &App) {
    if app.proposed_changes.is_empty() {
        return;
    }
    
    let area = centered_rect(90, 90, f.area());
    f.render_widget(Clear, area);
    
    // Split into: file list (left), diff view (right), status bar (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),      // Main content
            Constraint::Length(3),   // Status bar
        ].as_ref())
        .split(area);
    
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),  // File list
            Constraint::Percentage(70),  // Diff view
        ].as_ref())
        .split(chunks[0]);
    
    // File list (left panel)
    let file_list: Vec<ListItem> = app.proposed_changes
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
            let prefix = if i == app.plan_review_index { "> " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{} {}", prefix, status_icon, change.path), style)
            ]))
        })
        .collect();
    
    let file_list_widget = List::new(file_list)
        .block(Block::default().borders(Borders::ALL).title(" Files ").border_style(Style::default().fg(Color::Cyan)))
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(file_list_widget, main_chunks[0]);
    
    // Diff view (right panel) - show selected file
    if let Some(change) = app.proposed_changes.get(app.plan_review_index) {
        let diff_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunks[1]);
        
        let original = Paragraph::new(change.original.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Original ")
                .border_style(Style::default().fg(Color::Red)));
        let proposed = Paragraph::new(change.proposed.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Proposed ")
                .border_style(Style::default().fg(Color::Green)));
        
        f.render_widget(original, diff_chunks[0]);
        f.render_widget(proposed, diff_chunks[1]);
    }
    
    // Status bar
    let approved_count = app.proposed_changes.iter().filter(|c| c.status == ChangeStatus::Approved).count();
    let denied_count = app.proposed_changes.iter().filter(|c| c.status == ChangeStatus::Denied).count();
    let pending_count = app.proposed_changes.iter().filter(|c| c.status == ChangeStatus::Pending).count();
    
    let status_text = format!(
        " [↑/↓] Navigate | [A]pprove [D]eny | [Shift+A] Approve All | [Enter] Execute Approved | [Esc] Cancel | Pending: {} Approved: {} Denied: {} ",
        pending_count, approved_count, denied_count
    );
    let status = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[1]);
}

fn draw_servers_popup(f: &mut Frame, app: &App) {
    let accent_color = Color::Cyan;
    let fg_color = Color::White;
    
    let area = centered_rect(90, 80, f.area());
    f.render_widget(Clear, area);
    
    // Split into title, content, and status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Min(1),      // Content
            Constraint::Length(3),   // Status bar
        ].as_ref())
        .split(area);
    
    // Title
    let title = Paragraph::new("MCP Server Browser")
        .style(Style::default().fg(accent_color).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(accent_color)));
    f.render_widget(title, chunks[0]);
    
    // Content area - split into left (list) and right (details)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ].as_ref())
        .split(chunks[1]);
    
    // Left panel: Available MCP Servers list
    let available_servers: Vec<ListItem> = app.mcp_browser_items
        .iter()
        .enumerate()
        .map(|(i, (name, _desc, _))| {
            let is_installed = app.config.mcp.contains_key(name);
            let prefix = if i == app.mcp_browser_selected { "> " } else { "  " };
            let suffix = if is_installed { " [INSTALLED]" } else { "" };
            let style = if i == app.mcp_browser_selected {
                Style::default().fg(accent_color).add_modifier(Modifier::BOLD)
            } else if is_installed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(fg_color)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{}{}", prefix, name, suffix), style),
            ]))
        })
        .collect();
    
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(" Available Servers ")
        .border_style(Style::default().fg(accent_color));
    
    let list = List::new(available_servers)
        .block(list_block);
    f.render_widget(list, content_chunks[0]);
    
    // Right panel: Selected server details
    if let Some((name, desc, cmd)) = app.mcp_browser_items.get(app.mcp_browser_selected) {
        let is_installed = app.config.mcp.contains_key(name);
        let cmd_str = cmd.join(" ");
        
        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Yellow)),
                Span::styled(name, Style::default().fg(fg_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(desc.as_str()),
            Line::from(""),
            Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(cmd_str),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if is_installed { "Installed" } else { "Not installed" },
                    Style::default().fg(if is_installed { Color::Green } else { Color::Red })
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Note: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if is_installed { "Restart open_crust to use this server." } else { "Press [Enter] to install." },
                    Style::default().fg(fg_color)
                ),
            ]),
        ];
        
        let details_block = Block::default()
            .borders(Borders::ALL)
            .title(" Server Details ")
            .border_style(Style::default().fg(accent_color));
        
        let details_para = Paragraph::new(details)
            .block(details_block)
            .wrap(Wrap { trim: true });
        f.render_widget(details_para, content_chunks[1]);
    }
    
    // Status bar
    let status_text = if app.config.mcp.contains_key(
        &app.mcp_browser_items.get(app.mcp_browser_selected)
            .map(|(name, _, _)| name.clone())
            .unwrap_or_default()
    ) {
        " [↑/↓] Navigate | [Esc] Close "
    } else {
        " [↑/↓] Navigate | [Enter] Install | [Esc] Close "
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().bg(accent_color).fg(Color::Black))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

fn draw_command_palette(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 30, f.area());
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Min(1),      // Items
            Constraint::Length(3),   // Status
        ].as_ref())
        .split(area);
    
    // Title
    let title = Paragraph::new("Command Palette")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(title, chunks[0]);
    
    // Items
    let items = vec![
        ("Switch Provider", format!("Current: {:?}", app.config.provider)),
        ("Switch Model", format!("Current: {}", app.config.model)),
        ("Show Stats", "View usage statistics".to_string()),
        ("Clear Context", "Clear conversation history".to_string()),
        ("MCP Browser", "Manage MCP servers".to_string()),
    ];
    
    let menu_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (label, detail))| {
            let prefix = if i == app.command_palette_selected { "> " } else { "  " };
            let style = if i == app.command_palette_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}", prefix), style),
                Span::styled(format!("{}", label), style),
                Span::styled(format!("  {}", detail), style.dim()),
            ]))
        })
        .collect();
    
    let list = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title(" Commands "));
    f.render_widget(list, chunks[1]);
    
    // Status
    let status = Paragraph::new("[↑/↓] Navigate | [Enter] Select | [Esc] Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
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
    if s.starts_with('#') && s.len() == 7
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[1..3], 16),
            u8::from_str_radix(&s[3..5], 16),
            u8::from_str_radix(&s[5..7], 16),
        ) {
        return Color::Rgb(r, g, b);
    }
    Color::Reset
}
