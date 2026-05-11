use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::app::App;
use crate::ui::ThemeContext;

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    let mode_str = match app.mode {
        crate::app::Mode::Normal => "NORMAL",
        crate::app::Mode::Insert => "INSERT",
        crate::app::Mode::Review => "REVIEW",
        crate::app::Mode::Servers => "SERVERS",
        crate::app::Mode::SkillBrowser => "SKILLS",
        crate::app::Mode::CommandPalette => "PALETTE",
        crate::app::Mode::McpShowcase => "MCP SHOWCASE",
        crate::app::Mode::MissionControl => "MISSION CONTROL",
    };

    let vim_indicator = if app.vim_mode { " [VIM]" } else { "" };

    let status_bar = Paragraph::new(format!(
        "-- {} -- {} | Ctrl+B: Sidebar | Tab: Switch view",
        mode_str, vim_indicator,
    ))
    .style(Style::default().fg(theme.accent));
    f.render_widget(status_bar, area);
}
