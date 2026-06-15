//! TUI (Terminal UI) rendering — Ratatui-based interface
//!
//! Root UI module. Provides the main `draw()` entry point and shared
//! theme/color utilities. Sub-modules handle chat, layout, and popups.

mod chat;
mod layout;
mod popups;

use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
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
            bg: parse_color(theme.map(|t| t.background.as_str()).unwrap_or("#131211")),
            fg: parse_color(theme.map(|t| t.foreground.as_str()).unwrap_or("#b6b5b4")),
            accent: parse_color(theme.map(|t| t.accent.as_str()).unwrap_or("#f0eeee")),
            border: parse_color(theme.map(|t| t.border.as_str()).unwrap_or("#3c3b3a")),
        }
    }

    /// Derived error color — warm red derived from theme
    pub fn error(&self) -> Color {
        Color::Rgb(200, 80, 80)
    }

    /// Derived system notification color — warm amber
    pub fn system(&self) -> Color {
        Color::Rgb(180, 160, 80)
    }

    /// Derived dimmed ghost text color
    pub fn ghost(&self) -> Color {
        Color::Rgb(73, 72, 71)
    }

    /// Status bar mode-specific background for insert mode
    pub fn status_insert_bg(&self) -> Color {
        Color::Rgb(30, 50, 30)
    }

    /// Status bar mode-specific background for review mode
    pub fn status_review_bg(&self) -> Color {
        Color::Rgb(50, 40, 20)
    }

    /// Status bar mode-specific background for modal modes
    pub fn status_modal_bg(&self) -> Color {
        Color::Rgb(30, 30, 50)
    }

    /// Status bar default background
    pub fn status_default_bg(&self) -> Color {
        Color::Rgb(26, 25, 25)
    }

    /// Status bar foreground
    pub fn status_fg(&self) -> Color {
        Color::Rgb(240, 238, 238)
    }

    /// Shadow background for popup depth effect
    pub fn shadow(&self) -> Color {
        Color::Rgb(10, 9, 9)
    }

    /// Diff deletion background (subtle red)
    pub fn diff_delete_bg(&self) -> Color {
        Color::Rgb(60, 20, 20)
    }

    /// Diff deletion foreground
    pub fn diff_delete_fg(&self) -> Color {
        Color::Rgb(200, 130, 130)
    }

    /// Diff insertion background (subtle green)
    pub fn diff_insert_bg(&self) -> Color {
        Color::Rgb(20, 50, 20)
    }

    /// Diff insertion foreground
    pub fn diff_insert_fg(&self) -> Color {
        Color::Rgb(130, 200, 130)
    }

    /// Dimmed/muted text color
    pub fn dim(&self) -> Color {
        Color::Rgb(111, 109, 108)
    }

    /// Success/install indicator color
    pub fn success(&self) -> Color {
        Color::Green
    }

    /// Warning/highlight color
    pub fn warning(&self) -> Color {
        Color::Yellow
    }
}

pub fn parse_color(s: &str) -> Color {
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            6 => {
                // Standard 6-char hex: #RRGGBB
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return Color::Rgb(r, g, b);
                }
            }
            3 => {
                // Short 3-char hex: #RGB → expand to #RRGGBB
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..1].repeat(2), 16),
                    u8::from_str_radix(&hex[1..2].repeat(2), 16),
                    u8::from_str_radix(&hex[2..3].repeat(2), 16),
                ) {
                    return Color::Rgb(r, g, b);
                }
            }
            _ => {}
        }
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
        .enumerate()
        .map(|(i, t)| {
            let is_active = i == app.active_tab;
            let style = if is_active {
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            Line::from(Span::styled(format!(" {} ", t.name), style))
        })
        .collect();
    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Views ")
                .border_style(Style::default().fg(theme.accent)),
        )
        .select(app.active_tab)
        .highlight_style(Style::default().bg(theme.accent).fg(Color::Black));
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
        Mode::PluginBrowser => popups::draw_plugin_browser(f, app, &theme),
        Mode::CommandPalette => popups::draw_command_palette(f, app, &theme),
        Mode::Help => popups::draw_help_popup(f, app, &theme),
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

    // File picker overlay (renders on top of everything when active)
    if app.file_picker_active {
        popups::draw_file_picker(f, app, f.area(), &theme);
    }
}
