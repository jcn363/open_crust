//! Status bar rendering for the TUI
//!
//! Draws the bottom status bar showing mode (insert/normal), provider,
//! model, token count, and active agent/background task count.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

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
        crate::app::Mode::PluginBrowser => "PLUGINS",
        crate::app::Mode::CommandPalette => "PALETTE",
        crate::app::Mode::Help => "HELP",
        crate::app::Mode::McpShowcase => "MCP SHOWCASE",
        crate::app::Mode::MissionControl => "MISSION CONTROL",
    }
}

fn plan_mode_tag(app: &App) -> &'static str {
    match app.plan_mode {
        crate::app::PlanMode::Planning => " PLAN ",
        crate::app::PlanMode::Disabled => "",
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

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, _theme: &ThemeContext) {
    // Structure: [MODE] [PLAN] [Provider:Model] tokens:N/M cost:$X.XX tasks:N | keybind hints
    let mode_tag = format!(" {} ", mode_str(app.mode));
    let plan_tag = plan_mode_tag(app);
    let provider_tag = format!(
        " {}:{} ",
        provider_str(&app.config.provider),
        app.config.model
    );

    // Add token and cost information
    let token_tag = if let Some(_session_id) = &app.current_session_id {
        if let Some(budget) = app.token_budget.as_ref() {
            let usage_pct =
                (budget.current_tokens as f64 / budget.max_tokens as f64 * 100.0) as u32;
            format!(
                " tokens:{}/{} ({}%) cost:${:.2} ",
                budget.current_tokens, budget.max_tokens, usage_pct, budget.total_cost
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let task_tag = format!(" tasks:{} ", app.background_tasks.len());

    // Show keybinding hints based on mode
    let hints = match app.mode {
        crate::app::Mode::Normal => {
            " Ctrl+B:Sidebar  Tab:Switch  ?:Help  Ctrl+K:Palette  Ctrl+P:PlanMode  Ctrl+Shift+P:Plugins"
        }
        crate::app::Mode::Insert => " Esc:Normal  Enter:Send  ↑↓:History  Tab:Accept Ghost",
        crate::app::Mode::Review => {
            " ↑↓:Navigate  j/k:Scroll  u:ToggleView  A:Approve  D:Deny  Enter:Execute"
        }
        crate::app::Mode::Servers => " ↑↓:Navigate  Enter:Install  Type:Filter  Esc:Clear/Close",
        crate::app::Mode::Help => " Esc:Close",
        crate::app::Mode::SkillBrowser => " ↑↓:Navigate  Enter:Toggle  Esc/q:Close",
        crate::app::Mode::PluginBrowser => " ↑↓:Navigate  Enter:Toggle  Esc/q:Close",
        crate::app::Mode::CommandPalette => " ↑↓:Navigate  Enter:Select  Esc:Cancel",
        crate::app::Mode::McpShowcase => " ↑↓:Navigate  Enter:Toggle  Esc:Close",
        crate::app::Mode::MissionControl => " ↑↓:Navigate  Enter:Select  Esc:Close",
    };

    // Mode-specific background coloring for visual context
    let (mode_fg, mode_bg) = match app.mode {
        crate::app::Mode::Insert => (
            Color::Rgb(240, 238, 238),
            Color::Rgb(30, 50, 30), // subtle green for insert
        ),
        crate::app::Mode::Review => (
            Color::Rgb(240, 238, 238),
            Color::Rgb(50, 40, 20), // subtle amber for review
        ),
        crate::app::Mode::Servers
        | crate::app::Mode::SkillBrowser
        | crate::app::Mode::PluginBrowser
        | crate::app::Mode::CommandPalette
        | crate::app::Mode::McpShowcase
        | crate::app::Mode::MissionControl => (
            Color::Rgb(240, 238, 238),
            Color::Rgb(30, 30, 50), // subtle blue for modals
        ),
        _ => (
            Color::Rgb(240, 238, 238),
            Color::Rgb(26, 25, 25), // default dark
        ),
    };

    let status_bar = Paragraph::new(format!(
        "{}{}{}{}{}{}",
        mode_tag, plan_tag, provider_tag, token_tag, task_tag, hints,
    ))
    .style(Style::default().fg(mode_fg).bg(mode_bg));
    f.render_widget(status_bar, area);
}
