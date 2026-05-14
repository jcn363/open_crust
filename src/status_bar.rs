use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::app::App;
use crate::config::ProviderType;
use crate::ui::ThemeContext;

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    // Mode string
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

    // Input mode indicator
    let input_mode = match app.mode {
        crate::app::Mode::Insert => "[INS]",
        crate::app::Mode::CommandPalette => "[CMD]",
        _ => "",
    };

    let vim_indicator = if app.vim_mode { " [VIM]" } else { "" };

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
        "-- {} {}{} ({}) {} {} | Ctrl+B: Sidebar | Tab: Switch view",
        mode_str, provider_info, model_info, task_count, input_mode, vim_indicator
    ))
    .style(Style::default().fg(theme.accent));
    f.render_widget(status_bar, area);
}
