//! Command palette mode handler

use crate::app::{App, Message, Mode};
use crate::config::ProviderType;
use crate::event_loop::modes::types::{HandlerContext, ModeAction, ModeHandler};
use crossterm::event::{KeyCode, KeyEvent};

pub struct CommandPaletteHandler;

#[async_trait::async_trait]
impl ModeHandler for CommandPaletteHandler {
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
            KeyCode::Up if app.command_palette_selected > 0 => {
                app.command_palette_selected -= 1;
                ModeAction::Continue
            }
            KeyCode::Down if app.command_palette_selected < 3 => {
                // 4 items total (0-3)
                app.command_palette_selected += 1;
                ModeAction::Continue
            }
            KeyCode::Enter => {
                // Handle command palette selection
                match app.command_palette_selected {
                    0 => {
                        // Switch Provider
                        let providers = [
                            ProviderType::Ollama,
                            ProviderType::OpenRouter,
                            ProviderType::OpenAI,
                            ProviderType::Gemini,
                            ProviderType::Mistral,
                            ProviderType::Anthropic,
                            ProviderType::Groq,
                            ProviderType::TogetherAi,
                            ProviderType::Replicate,
                            ProviderType::DeepSeek,
                            ProviderType::LocalAi,
                            ProviderType::Unsloth,
                        ];
                        let current_idx = providers
                            .iter()
                            .position(|p| p == &app.config.provider)
                            .unwrap_or(0);
                        let next_idx = (current_idx + 1) % providers.len();
                        app.config.provider = providers[next_idx].clone();
                        app.config.save();
                        app.tabs[0].messages.push(Message::new(format!(
                            "Provider switched to {:?}",
                            app.config.provider
                        )));
                        app.mode = Mode::Normal;
                        ModeAction::SwitchMode(Mode::Normal)
                    }
                    1 => {
                        // Switch Model
                        app.tabs[0].messages.push(Message::new(format!(
                            "Model switching not fully implemented yet. Current model: {}",
                            app.config.model
                        )));
                        app.mode = Mode::Normal;
                        ModeAction::SwitchMode(Mode::Normal)
                    }
                    2 => {
                        // Clear Context
                        app.tabs[0].messages.clear();
                        app.tabs[0].messages.push(Message::new(String::from(
                            "Welcome to opencrust. Press 'i' to type, 'Tab' to switch tabs, 'Ctrl+Q' to quit.",
                        )));
                        app.history.clear();
                        app.save_history();
                        app.tabs[0]
                            .messages
                            .push(Message::new("Context cleared.".to_string()));
                        app.mode = Mode::Normal;
                        ModeAction::SwitchMode(Mode::Normal)
                    }
                    3 => {
                        // MCP Browser
                        app.mode = Mode::Servers;
                        ModeAction::SwitchMode(Mode::Servers)
                    }
                    _ => ModeAction::Continue,
                }
            }
            _ => ModeAction::Continue,
        }
    }
}
