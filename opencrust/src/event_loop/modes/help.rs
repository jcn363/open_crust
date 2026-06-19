//! Help mode handler

use crate::app::{App, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent};

pub struct HelpHandler;

#[async_trait::async_trait]
impl ModeHandler for HelpHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        _ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = Mode::Normal;
                ModeAction::SwitchMode(Mode::Normal)
            }
            _ => ModeAction::Continue,
        }
    }
}
