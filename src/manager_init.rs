//! Manager initialization for OpenCrust
//!
//! Extracted from `main.rs` to keep the entry point focused on startup logic.

use crate::config::Config;
use crate::custom_tools::CustomToolManager;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::skills::SkillManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Initialize all shared subsystem managers.
///
/// Returns a tuple of (mcp_manager, lsp_manager, skill_manager, custom_tool_manager).
pub async fn init_managers(
    config: &Config,
) -> (
    Arc<Mutex<McpManager>>,
    Arc<Mutex<LspManager>>,
    Arc<Mutex<SkillManager>>,
    Arc<Mutex<CustomToolManager>>,
) {
    let mcp_manager = Arc::new(Mutex::new(McpManager::new()));
    mcp_manager.lock().await.load_from_config(&config.mcp).await;

    let lsp_manager = Arc::new(Mutex::new(LspManager::new()));
    lsp_manager.lock().await.load_from_config(&config.lsp).await;

    let skill_manager = Arc::new(Mutex::new(SkillManager::new()));
    {
        let mut skills = skill_manager.lock().await;
        skills.discover();
    }

    let custom_tool_manager = Arc::new(Mutex::new(CustomToolManager::new()));
    {
        let mut custom = custom_tool_manager.lock().await;
        custom.discover();
    }

    (mcp_manager, lsp_manager, skill_manager, custom_tool_manager)
}

/// Spawn background model list refresh if enabled in config.
pub fn spawn_model_refresh(config: &Config) {
    if let Some(refresh_config) = &config.model_auto_refresh
        && refresh_config.enabled
    {
        let interval = refresh_config.interval_secs;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            loop {
                interval.tick().await;
                // Model refresh logic would go here
                // For now, this is a placeholder
            }
        });
    }
}

/// Apply Cinnamon theme detection to config.
pub fn apply_cinnamon_theme(config: &mut Config) {
    let info = crate::desktop::detection::get_cinnamon_info();
    if !info.theme.background.is_empty() {
        config.theme = Some(crate::config::ThemeConfig {
            background: info.theme.background.clone(),
            foreground: info.theme.foreground.clone(),
            accent: info.theme.accent.clone(),
            border: info.theme.border.clone(),
        });
    }
}
