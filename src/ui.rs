mod chat;
mod layout;
mod popups;

use ratatui::{
    Frame,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Tabs},
};

use crate::app::{App, Mode};
use crate::status_bar;

/// Centralized theme accessor — wraps config theme colors.
pub struct ThemeContext {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub border: Color,
}

impl ThemeContext {
    pub fn from_config(config: &crate::config::Config) -> Self {
        let theme = config.theme.as_ref();
        Self {
            bg: parse_color(theme.map(|t| t.background.as_str()).unwrap_or("#1e1e1e")),
            fg: parse_color(theme.map(|t| t.foreground.as_str()).unwrap_or("#ffffff")),
            accent: parse_color(theme.map(|t| t.accent.as_str()).unwrap_or("#007acc")),
            border: parse_color(theme.map(|t| t.border.as_str()).unwrap_or("#333333")),
        }
    }
}

fn parse_color(s: &str) -> Color {
    if s.starts_with('#')
        && s.len() == 7
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[1..3], 16),
            u8::from_str_radix(&s[3..5], 16),
            u8::from_str_radix(&s[5..7], 16),
        )
    {
        return Color::Rgb(r, g, b);
    }
    Color::Reset
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = ThemeContext::from_config(&app.config);

    // Background
    let bg_block = Block::default().style(Style::default().bg(theme.bg));
    f.render_widget(bg_block, f.area());

    let chunks = layout::main_layout(f.area());

    // Tabs
    let tab_titles: Vec<Line> = app
        .tabs
        .iter()
        .map(|t| Line::from(t.name.as_str()))
        .collect();
    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(" Views ")
                .border_style(Style::default().fg(theme.border)),
        )
        .select(app.active_tab)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(ratatui::style::Modifier::BOLD),
        );
    f.render_widget(tabs_widget, chunks[0]);

    let (sidebar_area, main_content_area) = layout::sidebar_layout(chunks[1], app.show_sidebar);

    // Sidebar
    if app.show_sidebar {
        chat::draw_sidebar(f, app, sidebar_area, &theme);
    }

    // Messages
    chat::draw_message_list(f, app, main_content_area, &theme);

    // Input
    chat::draw_input_area(f, app, chunks[2], &theme);

    // Status bar
    status_bar::draw_status_bar(f, app, chunks[3], &theme);

    // Modals / popups
    match app.mode {
        Mode::Review => popups::draw_review_popup(f, app, &theme),
        Mode::Servers => popups::draw_servers_popup(f, app, &theme),
        Mode::SkillBrowser => popups::draw_skill_browser(f, app, &theme),
        Mode::CommandPalette => popups::draw_command_palette(f, app, &theme),
        Mode::McpShowcase => {
            if let Some(ui) = app.mcp_showcase_ui.as_mut() {
                ui.render(f, f.area());
            }
        }
        Mode::MissionControl => {
            if let Some(ui) = app.mission_control_ui.as_mut() {
                ui.render(f, f.area());
            }
        }
        _ => {}
    }
}
