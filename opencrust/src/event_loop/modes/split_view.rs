//! Split view mode handler

use crate::app::{App, Mode, SplitViewMode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent};

pub struct SplitViewHandler;

#[async_trait::async_trait]
impl ModeHandler for SplitViewHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        _ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        match key.code {
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                ModeAction::SwitchMode(Mode::Normal)
            }
            KeyCode::Tab => {
                // Cycle through split view modes
                app.split_view_mode = match app.split_view_mode {
                    SplitViewMode::SideBySide => SplitViewMode::InlineUnified,
                    SplitViewMode::InlineUnified => SplitViewMode::InlineSplit,
                    SplitViewMode::InlineSplit => SplitViewMode::SideBySide,
                };
                let mode_name = match app.split_view_mode {
                    SplitViewMode::SideBySide => "Side by Side",
                    SplitViewMode::InlineUnified => "Unified Diff",
                    SplitViewMode::InlineSplit => "Split Diff",
                };
                app.tabs[0].messages.push(crate::app::Message::new(format!(
                    "Split view mode: {}",
                    mode_name
                )));
                ModeAction::Continue
            }
            KeyCode::Left => {
                app.split_left_scroll = app.split_left_scroll.saturating_add(1);
                ModeAction::Continue
            }
            KeyCode::Right => {
                app.split_right_scroll = app.split_right_scroll.saturating_add(1);
                ModeAction::Continue
            }
            KeyCode::Up => {
                app.split_left_scroll = app.split_left_scroll.saturating_sub(1);
                ModeAction::Continue
            }
            KeyCode::Down => {
                app.split_right_scroll = app.split_right_scroll.saturating_sub(1);
                ModeAction::Continue
            }
            KeyCode::Char('j') => {
                app.split_right_scroll = app.split_right_scroll.saturating_add(1);
                ModeAction::Continue
            }
            KeyCode::Char('k') => {
                app.split_right_scroll = app.split_right_scroll.saturating_sub(1);
                ModeAction::Continue
            }
            KeyCode::Char('q') => {
                app.mode = Mode::Normal;
                ModeAction::SwitchMode(Mode::Normal)
            }
            _ => ModeAction::Continue,
        }
    }
}
