//! Status bar rendering for the TUI
//!
//! Draws the bottom status bar showing mode (insert/normal), provider,
//! model, token count, active agent/background task count, and provider
//! fallback status with tooltip.

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
        ProviderType::Unsloth => "Unsloth",
        ProviderType::AzureOpenAi => "Azure",
        ProviderType::GitHubCopilot => "Copilot",
        ProviderType::Bedrock => "Bedrock",
        ProviderType::VertexAi => "Vertex",
        ProviderType::Perplexity => "Perplexity",
        ProviderType::Cohere => "Cohere",
        ProviderType::Cerebras => "Cerebras",
        ProviderType::AlibabaCloud => "Alibaba",
        ProviderType::VeniceAi => "Venice",
        ProviderType::Nvidia => "NVIDIA",
        ProviderType::FireworksAi => "Fireworks",
        ProviderType::SambaNova => "SambaNova",
        ProviderType::OctoAi => "OctoAI",
        ProviderType::Anyscale => "Anyscale",
        ProviderType::LambdaLabs => "Lambda",
        ProviderType::RunPod => "RunPod",
        ProviderType::Modal => "Modal",
        ProviderType::HuggingFace => "HuggingFace",
        ProviderType::LMStudio => "LMStudio",
        ProviderType::TGI => "TGI",
        ProviderType::VLLM => "vLLM",
        ProviderType::CustomOpenAi => "Custom",
    }
}

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeContext) {
    // Structure: [MODE] [PLAN] [Provider:Model] [FALLBACK] tokens:N/M cost:$X.XX tasks:N | keybind hints
    let mode_tag = format!(" {} ", mode_str(app.mode));
    let plan_tag = plan_mode_tag(app);
    let provider_tag = format!(
        " {}:{} ",
        provider_str(&app.config.provider),
        app.config.model
    );

    // Add fallback badge if a fallback occurred
    let fallback_tag = if let Some((ref provider, _timestamp)) = app.fallback_provider {
        format!(" ⚠ FALLBACK:{} ", provider)
    } else {
        String::new()
    };

    // Add token and cost information from live budget
    let token_tag = {
        if let Some(budget) = app.token_budget.as_ref() {
            let usage_pct =
                (budget.current_tokens as f64 / budget.max_tokens as f64 * 100.0) as u32;
            let warning = if budget.at_stop_threshold() {
                " [OVER]"
            } else if budget.at_warning_threshold() {
                " [WARN]"
            } else {
                ""
            };
            format!(
                " tokens:{}/{} ({}%)${:.4}{} ",
                budget.current_tokens, budget.max_tokens, usage_pct, budget.total_cost, warning
            )
        } else {
            String::new()
        }
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
        crate::app::Mode::Insert => (theme.status_fg(), theme.status_insert_bg()),
        crate::app::Mode::Review => (theme.status_fg(), theme.status_review_bg()),
        crate::app::Mode::Servers
        | crate::app::Mode::SkillBrowser
        | crate::app::Mode::PluginBrowser
        | crate::app::Mode::CommandPalette
        | crate::app::Mode::McpShowcase
        | crate::app::Mode::MissionControl => (theme.status_fg(), theme.status_modal_bg()),
        _ => (theme.status_fg(), theme.status_default_bg()),
    };

    // Use warning color for fallback badge
    let fallback_style = if app.fallback_provider.is_some() {
        Style::default().fg(theme.warning()).bg(mode_bg)
    } else {
        Style::default().fg(mode_fg).bg(mode_bg)
    };

    let status_text = format!(
        "{}{}{}{}{}{}",
        mode_tag, plan_tag, provider_tag, fallback_tag, token_tag, task_tag
    );
    let status_text_len = status_text.len();

    let status_bar = Paragraph::new(status_text).style(Style::default().fg(mode_fg).bg(mode_bg));
    f.render_widget(status_bar, area);

    // Render hints separately with fallback tooltip if active
    if let Some((ref provider, timestamp)) = app.fallback_provider {
        let elapsed = chrono::Utc::now().signed_duration_since(timestamp);
        let elapsed_secs = elapsed.num_seconds();
        let tooltip_text = format!(
            " ⚠ Fallback: {} (switched {}s ago) — Primary failed, using fallback provider",
            provider, elapsed_secs
        );
        let hints_with_tooltip = format!("{}{}", hints, tooltip_text);
        let hints_area = Rect {
            x: area.x + status_text_len as u16,
            y: area.y,
            width: area.width.saturating_sub(status_text_len as u16),
            height: 1,
        };
        if hints_area.width > 0 {
            let hints_widget = Paragraph::new(hints_with_tooltip).style(fallback_style);
            f.render_widget(hints_widget, hints_area);
        }
    }
}
