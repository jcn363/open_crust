//! Mission Control mode handler

use crate::app::{App, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::KeyEvent;

pub struct MissionControlHandler;

#[async_trait::async_trait]
impl ModeHandler for MissionControlHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        _ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        if let Some(ref mut ui) = app.mission_control_ui {
            // Refresh tasks from orchestrator bridge before handling input
            if let Some(ref tasks_arc) = app.orchestrator_tasks {
                ui.refresh_tasks(Some(tasks_arc));
            }
            if let crate::mission_control::MissionControlAction::ExitMode = ui.handle_key(key.code)
            {
                app.mode = Mode::Normal;
                ModeAction::SwitchMode(Mode::Normal)
            } else {
                ModeAction::Continue
            }
        } else {
            ModeAction::Continue
        }
    }
}
