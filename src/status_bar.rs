use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::app::App;
use crate::config::ProviderType;
use crate::ui::ThemeContext;

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    // Mode string (replaces redundant INS/VIM indicators — shown in input area title)
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

    // Get current model and provider info
    let model_info = format!(" {}", app.config.model);
    let provider_info = match app.config.provider {
        ProviderType::OpenRouter => "OpenRouter",
        ProviderType::Ollama => "Ollama",
        ProviderType::OpenAI => "OpenAI",
        ProviderType::Gemini => "Gemini",
        ProviderType::Mistral => "Mistral",
        ProviderType::Anthropic => "Anthropic",
        ProviderType::Groq => "Groq",
        ProviderType::TogetherAi => "TogetherAI",
        ProviderType::Replicate => "Replicate",
        ProviderType::DeepSeek => "DeepSeek",
        ProviderType::LocalAi => "LocalAI",
    };

    // Count background tasks
    let task_count = app.background_tasks.len();

    let status_bar = Paragraph::new(format!(
        " {} {}{} ({} tasks)  |  Ctrl+B: Sidebar  Tab: Switch  Ctrl+K: Palette  Ctrl+G: Mission",
        mode_str, provider_info, model_info, task_count,
    ))
    .style(Style::default().fg(theme.fg).bg(theme.accent));
    f.render_widget(status_bar, area);
}
