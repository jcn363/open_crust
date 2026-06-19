//! Application state and TUI orchestration
//!
//! Central state management for the terminal UI: holds all application
//! state (chat messages, tabs, input buffer, mode, sidebar, background
//! tasks). Handles event dispatch between UI layers, clipboard integration,
//! message history, and cursor movement within the input prompt.

mod background_tasks;
mod file_picker;
mod history;
mod input;
mod modes;
mod sidebar;
pub mod types;
mod vim;

pub use types::*;

use crate::config::Config;
use nucleo::Matcher;
use tokio::sync::mpsc;

/// Central application state for the TUI
///
/// Holds all UI state: tabs, messages, input buffer, cursor, mode,
/// sidebar state, background tasks, and clipboard. Methods handle
/// key events, message submission, history navigation, and more.
pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub input: String,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub should_quit: bool,
    pub prompt_tx: Option<mpsc::Sender<String>>,
    pub approval_tx: Option<mpsc::Sender<bool>>,
    pub waiting_for_approval: bool,
    pub proposed_changes: Vec<ProposedChange>,
    pub llm_client: crate::llm::LlmClient,
    pub mcp_input: String,
    pub show_sidebar: bool,
    pub background_tasks: Vec<BackgroundTask>, // Track background agent tasks
    pub background_task_tx: Option<tokio::sync::mpsc::Sender<String>>, // Send notifications for background task completion
    pub sidebar_items: Vec<String>,
    pub sidebar_selected: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    // MCP Browser state
    pub mcp_browser_items: Vec<(String, String, Vec<String>)>, // (name, description, command)
    pub mcp_browser_selected: usize,
    pub mcp_browser_scroll: usize,
    // Skill Browser state (Ctrl+Shift+K)
    pub skill_browser_items: Vec<(String, String, bool)>, // (name, description, active)
    pub skill_browser_selected: usize,
    pub skill_browser_scroll: usize,
    // Plugin Browser state (Ctrl+P)
    pub plugin_browser_items: Vec<(String, String, bool)>, // (name, description, enabled)
    pub plugin_browser_selected: usize,
    pub plugin_browser_scroll: usize,
    // Plan review index (for diff viewer)
    pub plan_review_index: usize, // Which file in the plan being reviewed
    // Scroll offset for diff panels in review mode
    pub plan_review_scroll: usize,
    // View mode toggle: false = side-by-side, true = unified diff
    pub review_show_unified: bool,
    // Plan mode state
    pub plan_mode: PlanMode,
    // Command palette state
    pub command_palette_selected: usize,
    // Input Prediction (Ghost Text)
    pub ghost_text: Option<String>,
    pub input_prediction_enabled: bool,
    pub last_input_time: Option<std::time::Instant>,
    // Vim Mode state
    pub vim_mode: bool,
    pub vim_cursor_pos: usize, // Cursor position in input field for vim editing
    // MCP Showcase state (Ctrl+M)
    pub mcp_showcase_ui: Option<crate::mcp_showcase::McpShowcaseUI>,
    // Mission Control state (Ctrl+G)
    pub mission_control_ui: Option<crate::mission_control::MissionControlUI>,
    /// Shared task state from orchestrator for live DAG visualization
    pub orchestrator_tasks:
        Option<std::sync::Arc<tokio::sync::RwLock<Vec<crate::orchestrator::task::Task>>>>,
    /// Scroll offset for the message list (positive = scroll up)
    pub message_scroll: usize,
    /// Token budget for current session
    pub token_budget: Option<crate::token_budget::TokenBudget>,
    // File picker state (@ fuzzy search)
    pub file_picker_active: bool,
    pub file_picker_query: String,
    pub file_picker_selected: usize,
    pub file_picker_results: Vec<String>,
    pub file_picker_scroll: usize,
    /// Cached full project file list (populated on file picker activation)
    cached_project_files: Vec<String>,
    /// Nucleo matcher for fast fuzzy matching
    file_matcher: Option<Matcher>,
    /// Debounce timer for file picker input
    file_picker_last_input: Option<std::time::Instant>,
    /// Custom commands manager (from .opencrust/commands/)
    pub custom_commands: crate::custom_commands::CustomCommandManager,
    /// Whether the UI needs a redraw (dirty flag for idle optimization)
    pub dirty: bool,
    /// Fallback provider state: (provider_name, timestamp) when a fallback occurred
    pub fallback_provider: Option<(String, chrono::DateTime<chrono::Utc>)>,
    /// Split view mode for diff rendering
    pub split_view_mode: SplitViewMode,
    /// Left pane content for split view
    pub split_left_content: Option<String>,
    /// Right pane content for split view
    pub split_right_content: Option<String>,
    /// Left pane scroll offset
    pub split_left_scroll: usize,
    /// Right pane scroll offset
    pub split_right_scroll: usize,
}

impl App {
    pub fn new(
        config: Config,
        prompt_tx: mpsc::Sender<String>,
        approval_tx: mpsc::Sender<bool>,
        background_task_tx: tokio::sync::mpsc::Sender<String>,
        llm_client: crate::llm::LlmClient,
    ) -> Self {
        let chat_tab = Tab {
            name: "Chat".to_string(),
            messages: vec![Message::new(String::from(
                "Welcome to OpenCrust. Press 'i' to type, 'Tab' to switch tabs, '?' for help, 'Ctrl+Q' to quit.",
            ))],
        };
        let tasks_tab = Tab {
            name: "Tasks".to_string(),
            messages: vec![Message::new(String::from("No tasks yet."))],
        };

        let mut app = Self {
            config,
            mode: Mode::Normal,
            input: String::new(),
            tabs: vec![chat_tab, tasks_tab],
            active_tab: 0,
            should_quit: false,
            prompt_tx: Some(prompt_tx),
            approval_tx: Some(approval_tx),
            waiting_for_approval: false,
            proposed_changes: Vec::new(),
            llm_client,
            mcp_input: String::new(),
            show_sidebar: true,
            sidebar_items: Vec::new(),
            sidebar_selected: 0,
            history: Vec::new(),
            history_index: None,
            // Input Prediction fields
            ghost_text: None,
            input_prediction_enabled: true,
            last_input_time: None,
            // MCP Browser initialization
            mcp_browser_items: vec![
                (
                    "github".to_string(),
                    "GitHub integration".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-github".to_string(),
                    ],
                ),
                (
                    "slack".to_string(),
                    "Slack integration".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-slack".to_string(),
                    ],
                ),
                (
                    "filesystem".to_string(),
                    "File system access".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                    ],
                ),
                (
                    "postgres".to_string(),
                    "PostgreSQL database".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-postgres".to_string(),
                    ],
                ),
                (
                    "google-drive".to_string(),
                    "Google Drive".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-google-drive".to_string(),
                    ],
                ),
                (
                    "git".to_string(),
                    "Git repository tools".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-git".to_string(),
                    ],
                ),
                (
                    "sqlite".to_string(),
                    "SQLite database".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-sqlite".to_string(),
                    ],
                ),
                (
                    "brave-search".to_string(),
                    "Brave search API".to_string(),
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-brave-search".to_string(),
                    ],
                ),
            ],
            mcp_browser_selected: 0,
            mcp_browser_scroll: 0,
            // Skill Browser initialization (populated in main.rs after skill discovery)
            skill_browser_items: Vec::new(),
            skill_browser_selected: 0,
            skill_browser_scroll: 0,
            // Plugin Browser initialization (populated in event_loop.rs)
            plugin_browser_items: Vec::new(),
            plugin_browser_selected: 0,
            plugin_browser_scroll: 0,
            // Plan review index (for diff viewer)
            plan_review_index: 0,
            plan_review_scroll: 0,
            review_show_unified: false,
            // Plan mode state
            plan_mode: PlanMode::Disabled,
            // Command palette state
            command_palette_selected: 0,
            // Background tasks initialization
            background_tasks: Vec::new(),
            background_task_tx: Some(background_task_tx),
            // Vim Mode state
            vim_mode: false,
            vim_cursor_pos: 0,
            // MCP Showcase state
            mcp_showcase_ui: None,
            // Mission Control state
            mission_control_ui: None,
            orchestrator_tasks: None,
            message_scroll: 0,
            // Token budget tracking
            token_budget: None,
            // File picker state (@ fuzzy search)
            file_picker_active: false,
            file_picker_query: String::new(),
            file_picker_selected: 0,
            file_picker_results: Vec::new(),
            file_picker_scroll: 0,
            cached_project_files: Vec::new(),
            file_matcher: None,
            file_picker_last_input: None,
            // Custom commands manager
            custom_commands: crate::custom_commands::CustomCommandManager::new(),
            // Dirty flag — starts true to force initial render
            dirty: true,
            // Fallback provider tracking
            fallback_provider: None,
            // Split view state
            split_view_mode: SplitViewMode::default(),
            split_left_content: None,
            split_right_content: None,
            split_left_scroll: 0,
            split_right_scroll: 0,
        };
        app.load_history();
        // Discover custom commands from .opencrust/commands/
        app.custom_commands.discover();
        // Restore last session if available
        if let Some(msg) = app.restore_session() {
            app.tabs[0].messages.push(Message::new(msg));
        }
        // First-run onboarding: show setup guidance if no config file exists
        {
            let config_dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".config/opencrust");
            let config_file = config_dir.join("config.json");
            if !config_file.exists() {
                let onboarding_msg = Message::new(
                    "Welcome to OpenCrust! First-time setup:\n\n\
                     1. Set your provider: /provider <name> (ollama, openrouter, openai, etc.)\n\
                     2. Set your model: /model <model-name>\n\
                     3. Set a token budget: /budget <max_tokens> (e.g., /budget 1000000)\n\
                     4. View costs: /cost\n\
                     5. Set fallback providers: /fallback openai,anthropic,groq\n\n\
                     Key bindings: i=Insert, Tab=Switch tabs, Ctrl+P=Plan Mode, Ctrl+G=Mission Control, ?=Help"
                        .to_string(),
                );
                app.tabs[0].messages.push(onboarding_msg);
            }
        }
        app
    }
}

#[cfg(test)]
mod tests;
