//! Shared types for mode handlers

use crate::app::{App, Mode};
use crate::clipboard::ClipboardManager;
use crate::llm::LlmClient;
use crate::orchestrator::task::Task;
use crate::plugins::PluginManager;
use crate::skills::SkillManager;
use crossterm::event::KeyEvent;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Action to take after handling a key event in a mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeAction {
    /// Continue in the current mode
    Continue,
    /// Exit the current mode, return to Normal
    #[allow(dead_code)]
    ExitMode,
    /// Switch to a different mode
    SwitchMode(Mode),
    /// Quit the application
    #[allow(dead_code)]
    Quit,
}

/// Context passed to mode handlers containing shared dependencies
pub struct HandlerContext<'a> {
    pub skill_manager: Arc<Mutex<SkillManager>>,
    pub plugin_manager: Arc<Mutex<PluginManager>>,
    pub clipboard: Arc<Mutex<ClipboardManager>>,
    #[allow(dead_code)]
    pub llm_client: &'a LlmClient,
    #[allow(dead_code)]
    pub orchestrator_tasks: Option<Arc<RwLock<Vec<Task>>>>,
}

impl<'a> HandlerContext<'a> {
    pub fn new(
        skill_manager: Arc<Mutex<SkillManager>>,
        plugin_manager: Arc<Mutex<PluginManager>>,
        clipboard: Arc<Mutex<ClipboardManager>>,
        llm_client: &'a LlmClient,
        orchestrator_tasks: Option<Arc<RwLock<Vec<Task>>>>,
    ) -> Self {
        Self {
            skill_manager,
            plugin_manager,
            clipboard,
            llm_client,
            orchestrator_tasks,
        }
    }
}

/// Trait for mode-specific key handling
#[async_trait::async_trait]
pub trait ModeHandler: Send {
    /// Handle a key event in this mode
    /// Returns the action to take after handling
    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        ctx: &mut HandlerContext<'_>,
    ) -> ModeAction;
}
