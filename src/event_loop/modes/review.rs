//! Review mode handler

use crate::app::{App, Message, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct ReviewHandler;

#[async_trait::async_trait]
impl ModeHandler for ReviewHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        _ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        match key.code {
            // Navigation between files
            KeyCode::Up if app.plan_review_index > 0 => {
                app.plan_review_index -= 1;
                app.plan_review_scroll = 0;
                ModeAction::Continue
            }
            KeyCode::Down if app.plan_review_index + 1 < app.proposed_changes.len() => {
                app.plan_review_index += 1;
                app.plan_review_scroll = 0;
                ModeAction::Continue
            }
            // Scroll diff view (j/k)
            KeyCode::Char('j') => {
                app.plan_review_scroll = app.plan_review_scroll.saturating_add(1);
                ModeAction::Continue
            }
            KeyCode::Char('k') => {
                app.plan_review_scroll = app.plan_review_scroll.saturating_sub(1);
                ModeAction::Continue
            }
            // Toggle unified / side-by-side view ('u')
            KeyCode::Char('u') => {
                app.review_show_unified = !app.review_show_unified;
                app.plan_review_scroll = 0;
                ModeAction::Continue
            }
            // Approve current file
            KeyCode::Char('a') => {
                if let Some(change) = app.proposed_changes.get_mut(app.plan_review_index) {
                    change.status = crate::app::ChangeStatus::Approved;
                }
                ModeAction::Continue
            }
            // Deny current file
            KeyCode::Char('d') => {
                if let Some(change) = app.proposed_changes.get_mut(app.plan_review_index) {
                    change.status = crate::app::ChangeStatus::Denied;
                }
                ModeAction::Continue
            }
            // Approve all files (Shift+A)
            KeyCode::Char('A') if key.modifiers == KeyModifiers::SHIFT => {
                for change in &mut app.proposed_changes {
                    change.status = crate::app::ChangeStatus::Approved;
                }
                ModeAction::Continue
            }
            // Execute approved changes
            KeyCode::Enter => {
                // Drain approved changes without cloning
                let all_changes = std::mem::take(&mut app.proposed_changes);
                let mut approved_count = 0usize;
                for change in all_changes {
                    if change.status == crate::app::ChangeStatus::Approved {
                        if let Err(e) = std::fs::write(&change.path, &change.proposed) {
                            app.tabs[0].messages.push(Message::new(format!(
                                "Error writing {}: {}",
                                change.path, e
                            )));
                        } else {
                            app.tabs[0]
                                .messages
                                .push(Message::new(format!("Applied: {}", change.path)));
                        }
                        approved_count += 1;
                    }
                }

                app.plan_review_index = 0;
                app.mode = Mode::Normal;
                app.tabs[0].messages.push(Message::new(format!(
                    "Executed {} approved changes",
                    approved_count
                )));
                ModeAction::SwitchMode(Mode::Normal)
            }
            // Cancel (Esc)
            KeyCode::Esc => {
                app.proposed_changes.clear();
                app.plan_review_index = 0;
                app.mode = Mode::Normal;
                app.tabs[0]
                    .messages
                    .push(Message::new(String::from("Plan cancelled")));
                ModeAction::SwitchMode(Mode::Normal)
            }
            _ => ModeAction::Continue,
        }
    }
}
