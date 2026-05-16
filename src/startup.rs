//! Shared startup initialization for OpenCrust subsystems
//!
//! Provides helper functions for manager creation, desktop environment
//! detection, and background tasks that are used by both headless
//! and interactive modes.

use std::sync::Arc;
use tokio::sync::Mutex;

/// Detect Cinnamon desktop environment and apply its theme colors.
///
/// Logs detection info to stderr for debugging. If no custom theme
/// is configured and Cinnamon is detected, fills in the theme from
/// the Cinnamon theme (background, foreground, accent, border).
pub fn apply_cinnamon_theme(config: &mut crate::config::Config) {
    let cinnamon_info = crate::desktop::detection::get_cinnamon_info();
    if cinnamon_info.desktop.is_cinnamon() {
        eprintln!(
            "[Desktop] Detected: {} {}",
            cinnamon_info.desktop,
            cinnamon_info.version.as_deref().unwrap_or("")
        );
    }

    if config.theme.is_none() && cinnamon_info.desktop.is_cinnamon() {
        config.theme = Some(crate::config::ThemeConfig {
            background: cinnamon_info.theme.background.clone(),
            foreground: cinnamon_info.theme.foreground.clone(),
            accent: cinnamon_info.theme.accent.clone(),
            border: cinnamon_info.theme.border.clone(),
        });
    }
}

/// Start a background task to refresh the model list on startup.
///
/// Fetches available models from the configured provider and logs
/// the result. Falls back to bundled defaults on fetch failure.
pub fn spawn_model_refresh(config: &crate::config::Config) {
    if let Some(ref auto_refresh) = config.model_auto_refresh
        && auto_refresh.enabled
    {
        let refresh_config = config.clone();
        tokio::spawn(async move {
            let fetcher = crate::models::ModelFetcher::new();
            let provider_str = refresh_config.provider.to_string();
            let models = fetcher.fetch(&provider_str, None, None).await;
            if models.is_empty() {
                let defaults = crate::models::bundled_default_models();
                if defaults.contains_key(&provider_str) {
                    eprintln!(
                        "[Models] Using bundled default model list for {}",
                        provider_str
                    );
                }
            } else {
                eprintln!(
                    "[Models] Refreshed {} model list ({} models)",
                    provider_str,
                    models.len()
                );
            }
        });
    }
}

/// Create and initialize the four shared subsystem managers.
///
/// Each manager is wrapped in `Arc<Mutex<>>` for concurrent access
/// from both the TUI event loop and any spawned sub-agent tasks.
#[allow(clippy::type_complexity)]
pub async fn init_managers(
    config: &crate::config::Config,
) -> (
    Arc<Mutex<crate::mcp::McpManager>>,
    Arc<Mutex<crate::lsp::LspManager>>,
    Arc<Mutex<crate::skills::SkillManager>>,
    Arc<Mutex<crate::custom_tools::CustomToolManager>>,
) {
    let mcp_manager = Arc::new(Mutex::new(crate::mcp::McpManager::new()));
    mcp_manager.lock().await.load_from_config(&config.mcp).await;

    let lsp_manager = Arc::new(Mutex::new(crate::lsp::LspManager::new()));
    lsp_manager.lock().await.load_from_config(&config.lsp).await;

    let skill_manager = Arc::new(Mutex::new(crate::skills::SkillManager::new()));
    {
        let mut skills = skill_manager.lock().await;
        skills.discover();
    }

    let custom_tool_manager = Arc::new(Mutex::new(crate::custom_tools::CustomToolManager::new()));
    {
        let mut custom = custom_tool_manager.lock().await;
        custom.discover();
    }

    (mcp_manager, lsp_manager, skill_manager, custom_tool_manager)
}
