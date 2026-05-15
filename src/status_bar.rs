//! Status bar rendering for the TUI
//!
//! Draws the bottom status bar showing mode (insert/normal), provider,
//! model, token count, and active agent/background task count.

use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::app::App;
use crate::config::ProviderType;
use crate::ui::ThemeContext;

fn mode_str(mode: crate::app::Mode) -> &'static str {
    match mode {
        crate::app::Mode::Normal => "NORMAL",
        crate::app::Mode::Insert => "INSERT",
        crate::app::Mode::Review => "REVIEW",
        crate::app::Mode::Servers => "SERVERS",
        crate::app::Mode::SkillBrowser => "SKILLS",
        crate::app::Mode::CommandPalette => "PALETTE",
        crate::app::Mode::Help => "HELP",
        crate::app::Mode::McpShowcase => "MCP SHOWCASE",
        crate::app::Mode::MissionControl => "MISSION CONTROL",
    }
}

fn provider_str(provider: &ProviderType) -> &'static str {
    match provider {
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
    }
}

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    // Structure: [MODE] [Provider:Model] tasks:N | keybind hints
    let mode_tag = format!(" {} ", mode_str(app.mode));
    let provider_tag = format!(
        " {}:{} ",
        provider_str(&app.config.provider),
        app.config.model
    );
    let task_tag = format!(" tasks:{} ", app.background_tasks.len());

    // Show keybinding hints based on mode
    let hints = match app.mode {
        crate::app::Mode::Normal => " Ctrl+B:Sidebar  Tab:Switch  ?:Help  Ctrl+K:Palette",
        crate::app::Mode::Insert => " Esc:Normal  Enter:Send  ↑↓:History",
        crate::app::Mode::Review => " ↑↓:Navigate  A:Approve  D:Deny  Enter:Execute",
        crate::app::Mode::Servers => " ↑↓:Navigate  Enter:Install  Esc:Close",
        crate::app::Mode::Help => " Esc:Close",
        _ => "",
    };

    let status_bar = Paragraph::new(format!("{}{}{}{}", mode_tag, provider_tag, task_tag, hints,))
        .style(Style::default().fg(theme.fg).bg(theme.accent));
    f.render_widget(status_bar, area);
}
