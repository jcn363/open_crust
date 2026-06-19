//! Insert mode handler

use crate::app::{App, Message, Mode};
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crate::event_loop::share;
use crossterm::event::{KeyCode, KeyEvent};

pub struct InsertHandler;

#[async_trait::async_trait]
impl ModeHandler for InsertHandler {
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        ctx: &mut HandlerContext<'_>,
    ) -> ModeAction {
        // Get submit keys from config
        let keybinds = app.config.tui.as_ref().map(|t| &t.keybinds);
        let submit_keys = keybinds
            .map(|k| k.input_submit.clone())
            .unwrap_or_else(|| "return".to_string());

        if crate::event_loop::keybinds::check_key_match(&key, &submit_keys) {
            // Handle slash commands before submitting
            let input_trimmed = app.input.trim();
            if input_trimmed == "/share" {
                let share_link = share::share_conversation(app);
                if let Some(link) = share_link {
                    let _ = ctx.clipboard.lock().await.copy(&link.file_path);
                    app.tabs[0].messages.push(Message::new(format!(
                        "Conversation shared: {} (id: {})\nPath copied to clipboard.",
                        link.tab_name, link.id
                    )));
                } else {
                    app.tabs[0].messages.push(Message::new(
                        "Error: Failed to share conversation.".to_string(),
                    ));
                }
                app.input.clear();
                ModeAction::Continue
            } else if input_trimmed == "/format" {
                let msg = app.format_current_file();
                app.tabs[0].messages.push(Message::new(msg));
                app.input.clear();
                ModeAction::Continue
            } else if input_trimmed.starts_with("/format ") {
                let path = input_trimmed.trim_start_matches("/format ").trim();
                match crate::formatters::format_file(std::path::Path::new(path)) {
                    Ok(_) => {
                        app.tabs[0]
                            .messages
                            .push(Message::new(format!("Formatted {}", path)));
                    }
                    Err(e) => {
                        app.tabs[0].messages.push(Message::new(e));
                    }
                }
                app.input.clear();
                ModeAction::Continue
            } else if input_trimmed.starts_with('/') {
                // Check for custom commands
                let cmd_name = input_trimmed
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                let args = input_trimmed
                    .trim_start_matches('/')
                    .strip_prefix(cmd_name)
                    .unwrap_or("")
                    .trim();
                if app.custom_commands.has_command(cmd_name) {
                    match app.custom_commands.execute_command(cmd_name, args) {
                        Ok(output) => {
                            app.tabs[0]
                                .messages
                                .push(Message::new(format!("/{}: {}", cmd_name, output)));
                        }
                        Err(e) => {
                            app.tabs[0]
                                .messages
                                .push(Message::new(format!("/{} error: {}", cmd_name, e)));
                        }
                    }
                    app.input.clear();
                    ModeAction::Continue
                } else {
                    app.submit_message();
                    ModeAction::Continue
                }
            } else {
                app.submit_message();
                ModeAction::Continue
            }
        } else if app.vim_mode {
            // Vim Mode input editing
            match key.code {
                KeyCode::Esc => {
                    if app.ghost_text.is_some() {
                        app.clear_ghost_text();
                    } else {
                        app.enter_normal_mode();
                    }
                    ModeAction::SwitchMode(Mode::Normal)
                }
                KeyCode::Backspace => {
                    app.handle_backspace();
                    ModeAction::Continue
                }
                // Vim navigation (specific chars BEFORE general Char(c))
                KeyCode::Char('h') => {
                    app.move_cursor_left();
                    ModeAction::Continue
                }
                KeyCode::Char('l') => {
                    app.move_cursor_right();
                    ModeAction::Continue
                }
                KeyCode::Char('w') => {
                    app.move_to_next_word();
                    ModeAction::Continue
                }
                KeyCode::Char('b') => {
                    app.move_to_prev_word();
                    ModeAction::Continue
                }
                KeyCode::Char('0') => {
                    app.move_to_line_start();
                    ModeAction::Continue
                }
                KeyCode::Char('$') => {
                    app.move_to_line_end();
                    ModeAction::Continue
                }
                KeyCode::Char('a') => {
                    app.move_cursor_right();
                    ModeAction::Continue
                }
                KeyCode::Char('d') => {
                    app.delete_line();
                    ModeAction::Continue
                }
                KeyCode::Char('c') => {
                    app.delete_line();
                    ModeAction::Continue
                }
                KeyCode::Char('y') => {
                    let mut clipboard = ctx.clipboard.lock().await;
                    let _ = app.yank_line(&mut clipboard);
                    ModeAction::Continue
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                    // Update last input time for prediction
                    app.last_input_time = Some(std::time::Instant::now());
                    ModeAction::Continue
                }
                _ => ModeAction::Continue,
            }
        } else {
            // Normal insert mode
            match key.code {
                KeyCode::Esc => {
                    app.enter_normal_mode();
                    ModeAction::SwitchMode(Mode::Normal)
                }
                KeyCode::Backspace => {
                    app.handle_backspace();
                    ModeAction::Continue
                }
                KeyCode::Left => {
                    app.move_cursor_left();
                    ModeAction::Continue
                }
                KeyCode::Right => {
                    app.move_cursor_right();
                    ModeAction::Continue
                }
                KeyCode::Home => {
                    app.move_to_line_start();
                    ModeAction::Continue
                }
                KeyCode::End => {
                    app.move_to_line_end();
                    ModeAction::Continue
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                    // Update last input time for prediction
                    app.last_input_time = Some(std::time::Instant::now());
                    ModeAction::Continue
                }
                _ => ModeAction::Continue,
            }
        }
    }
}
