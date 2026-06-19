use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::helpers::*;
use crate::app::App;
use crate::ui::ThemeContext;
use crate::ui::layout::centered_rect;

pub fn draw_command_palette(f: &mut Frame, app: &App, theme: &ThemeContext) {
    let area = centered_rect(60, 35, f.area());
    render_popup_shadow(f, area, theme);
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
                Style::default().fg(theme.warning())
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

pub fn draw_help_popup(f: &mut Frame, _app: &App, theme: &ThemeContext) {
    let area = centered_rect(65, 65, f.area());
    render_popup_shadow(f, area, theme);
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
        Line::from("  /share-list     List share links"),
        Line::from("  /diff <file>    Open file in split view"),
        Line::from("  /edit <file>    Open in external editor"),
        Line::from("  /memory <cmd>   Auto memory (remember/recall/forget/list)"),
        Line::from("  /agent <cmd>    Recursive agents (spawn/status/tree)"),
        Line::from("  /auth <cmd>     Auth login (copilot/chatgpt/status/clear)"),
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
