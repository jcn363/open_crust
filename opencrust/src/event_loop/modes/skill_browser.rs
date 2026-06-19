//! Skill browser mode handler

use crate::app::{App, Message, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent};

pub struct SkillBrowserHandler;

#[async_trait::async_trait]
impl ModeHandler for SkillBrowserHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = Mode::Normal;
                ModeAction::SwitchMode(Mode::Normal)
            }
            KeyCode::Up => {
                if app.skill_browser_selected > 0 {
                    app.skill_browser_selected -= 1;
                }
                // Adjust scroll if needed
                if app.skill_browser_selected < app.skill_browser_scroll {
                    app.skill_browser_scroll = app.skill_browser_selected;
                }
                ModeAction::Continue
            }
            KeyCode::Down => {
                if app.skill_browser_selected < app.skill_browser_items.len() - 1 {
                    app.skill_browser_selected += 1;
                }
                // Adjust scroll if needed (assuming 20 visible items)
                if app.skill_browser_selected >= app.skill_browser_scroll + 20 {
                    app.skill_browser_scroll = app.skill_browser_selected - 19;
                }
                ModeAction::Continue
            }
            KeyCode::Enter => {
                // Toggle skill active state
                if let Some((name, _, active)) =
                    app.skill_browser_items.get_mut(app.skill_browser_selected)
                {
                    let new_active = !*active;
                    *active = new_active;

                    // Update skill_manager (clone before moving into async block)
                    let skill_name = name.clone();
                    let sm = ctx.skill_manager.clone();
                    tokio::spawn(async move {
                        let mut skills = sm.lock().await;
                        if new_active {
                            let _ = skills.activate_skill(skill_name.as_str());
                        } else {
                            let _ = skills.deactivate_skill(skill_name.as_str());
                        }
                    });

                    let status = if new_active {
                        "activated"
                    } else {
                        "deactivated"
                    };
                    app.tabs[0]
                        .messages
                        .push(Message::new(format!("System: Skill '{}' {}", name, status)));
                }
                ModeAction::Continue
            }
            _ => ModeAction::Continue,
        }
    }
}
