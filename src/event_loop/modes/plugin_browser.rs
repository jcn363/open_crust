//! Plugin browser mode handler

use crate::app::{App, Message, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent};

pub struct PluginBrowserHandler;

#[async_trait::async_trait]
impl ModeHandler for PluginBrowserHandler {
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
                if app.plugin_browser_selected > 0 {
                    app.plugin_browser_selected -= 1;
                }
                if app.plugin_browser_selected < app.plugin_browser_scroll {
                    app.plugin_browser_scroll = app.plugin_browser_selected;
                }
                ModeAction::Continue
            }
            KeyCode::Down => {
                if app.plugin_browser_selected < app.plugin_browser_items.len() - 1 {
                    app.plugin_browser_selected += 1;
                }
                if app.plugin_browser_selected >= app.plugin_browser_scroll + 20 {
                    app.plugin_browser_scroll = app.plugin_browser_selected - 19;
                }
                ModeAction::Continue
            }
            KeyCode::Enter => {
                if let Some((name, _, enabled)) = app
                    .plugin_browser_items
                    .get_mut(app.plugin_browser_selected)
                {
                    let new_enabled = !*enabled;
                    *enabled = new_enabled;

                    // Update plugin_manager
                    let plugin_name = name.clone();
                    let pm = ctx.plugin_manager.clone();
                    tokio::spawn(async move {
                        let mut plugins = pm.lock().await;
                        if new_enabled {
                            let _ = plugins.enable(&plugin_name);
                        } else {
                            let _ = plugins.disable(&plugin_name);
                        }
                    });

                    let status = if new_enabled { "enabled" } else { "disabled" };
                    app.tabs[0].messages.push(Message::new(format!(
                        "System: Plugin '{}' {}",
                        name, status
                    )));
                }
                ModeAction::Continue
            }
            _ => ModeAction::Continue,
        }
    }
}
