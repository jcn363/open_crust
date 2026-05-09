use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::Paragraph,
};

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
            app.config.model,
            total_tokens,
            context_budget,
            context_percent,
            s.total_cost
        )
    } else {
        format!(" | 🤖 {} ", app.config.model)
    };

    let vim_indicator = if app.vim_mode { " [VIM]" } else { "" };
    let status_bar = Paragraph::new(format!(
        "-- {} -- {} | Ctrl+B: Sidebar | Tab: Switch view{}",
        mode_str, vim_indicator, stats_str
    ))
    .style(Style::default().fg(theme.accent));
    f.render_widget(status_bar, area);
}
