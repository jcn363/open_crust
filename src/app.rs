//! Application state and TUI orchestration
//!
//! Central state management for the terminal UI: holds all application
//! state (chat messages, tabs, input buffer, mode, sidebar, background
//! tasks). Handles event dispatch between UI layers, clipboard integration,
//! message history, and cursor movement within the input prompt.

use crate::config::Config;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Review,
    Servers,
    SkillBrowser,   // Ctrl+Shift+K skill browser
    PluginBrowser,  // Ctrl+P plugin browser
    CommandPalette, // Ctrl+K command palette
    Help,           // ? key help
    McpShowcase,    // Ctrl+M MCP Showcase
    MissionControl, // Ctrl+G Mission Control
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PlanMode {
    #[default]
    Disabled, // Normal execution
    Planning, // LLM is planning
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChangeStatus {
    Pending,
    Approved,
    Denied,
}

#[derive(Clone, Debug)]
pub struct ProposedChange {
    pub path: String,
    pub original: String,
    pub proposed: String,
    pub status: ChangeStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug)]
pub struct BackgroundTask {
    pub id: String,
    pub prompt: String,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Message {
    pub fn new(content: String) -> Self {
        Self {
            content,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub name: String,
    pub messages: Vec<Message>,
}

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
    /// Current session ID for token tracking
    pub current_session_id: Option<String>,
    // File picker state (@ fuzzy search)
    pub file_picker_active: bool,
    pub file_picker_query: String,
    pub file_picker_selected: usize,
    pub file_picker_results: Vec<String>,
    pub file_picker_scroll: usize,
    /// Cached full project file list (populated on file picker activation)
    cached_project_files: Vec<String>,
    /// Custom commands manager (from .opencrust/commands/)
    pub custom_commands: crate::custom_commands::CustomCommandManager,
    /// Whether the UI needs a redraw (dirty flag for idle optimization)
    pub dirty: bool,
}

impl App {
    /// Check if we should trigger input prediction (300ms debounce)
    pub fn should_trigger_prediction(&self) -> bool {
        if !self.input_prediction_enabled {
            return false;
        }
        if let Some(last_time) = self.last_input_time {
            last_time.elapsed() >= std::time::Duration::from_millis(300)
        } else {
            false
        }
    }

    /// Clear ghost text and reset prediction state
    pub fn clear_ghost_text(&mut self) {
        self.ghost_text = None;
        self.last_input_time = None;
    }

    /// Mark the UI as needing a redraw
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Activate the file picker with an initial query string.
    /// Caches the full project file list and filters by the query.
    pub fn activate_file_picker(&mut self, query: String) {
        self.file_picker_active = true;
        self.file_picker_query = query.clone();
        self.file_picker_selected = 0;
        self.file_picker_scroll = 0;
        // Cache the full file list on first activation
        if self.cached_project_files.is_empty() {
            self.cached_project_files = self.collect_all_project_files();
        }
        self.file_picker_results = self.filter_project_files(&query);
    }

    /// Deactivate the file picker without selecting a file.
    pub fn cancel_file_picker(&mut self) {
        self.file_picker_active = false;
        self.file_picker_query.clear();
        self.file_picker_results.clear();
    }

    /// Confirm the current selection and insert the file path into input.
    pub fn confirm_file_picker(&mut self) -> Option<String> {
        if let Some(path) = self.file_picker_results.get(self.file_picker_selected) {
            let selected = path.clone();
            self.cancel_file_picker();
            Some(selected)
        } else {
            None
        }
    }

    /// Format the currently selected file in the sidebar.
    /// Returns a message describing the result.
    pub fn format_current_file(&mut self) -> String {
        if self.sidebar_items.is_empty() || self.sidebar_selected >= self.sidebar_items.len() {
            return "No file selected in sidebar".to_string();
        }
        let path = &self.sidebar_items[self.sidebar_selected];
        match crate::formatters::format_file(std::path::Path::new(path)) {
            Ok(_) => format!("Formatted {}", path),
            Err(e) => e,
        }
    }

    /// Collect all files in the project (no filter). Used to populate cache.
    fn collect_all_project_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(".")
            .into_iter()
            .filter_entry(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| !n.starts_with('.'))
                    .unwrap_or(false)
            })
            .flatten()
        {
            if entry.file_type().is_file() {
                if let Some(path) = entry.path().to_str() {
                    // Skip binary/large files
                    if path.ends_with(".git")
                        || path.contains("/target/")
                        || path.contains("\\target\\")
                    {
                        continue;
                    }
                    files.push(path.to_string());
                }
            }
        }
        files.sort();
        files
    }

    /// Filter cached project files by query (fuzzy match). No filesystem access.
    pub(crate) fn filter_project_files(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return self.cached_project_files.iter().take(50).cloned().collect();
        }
        let query_lower = query.to_lowercase();
        let mut scored: Vec<_> = self
            .cached_project_files
            .iter()
            .filter_map(|f| {
                let score = fuzzy_score(f, &query_lower);
                if score > 0 {
                    Some((score, f.clone()))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by_key(|b| std::cmp::Reverse(b.0));
        scored.truncate(50);
        scored.into_iter().map(|(_, f)| f).collect()
    }
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
            current_session_id: None,
            // File picker state (@ fuzzy search)
            file_picker_active: false,
            file_picker_query: String::new(),
            file_picker_selected: 0,
            file_picker_results: Vec::new(),
            file_picker_scroll: 0,
            cached_project_files: Vec::new(),
            // Custom commands manager
            custom_commands: crate::custom_commands::CustomCommandManager::new(),
            // Dirty flag — starts true to force initial render
            dirty: true,
        };
        app.load_history();
        // Discover custom commands from .opencrust/commands/
        app.custom_commands.discover();
        app
    }

    /// Spawn a background task with the given prompt
    pub fn spawn_background_task(&mut self, prompt: String) {
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let task = BackgroundTask {
            id: task_id.clone(),
            prompt: prompt.clone(),
            status: TaskStatus::Running,
            result: None,
            started_at: now,
        };

        self.background_tasks.push(task);

        if let Some(tx) = &self.background_task_tx {
            let tx_clone = tx.clone();
            let llm = self.llm_client.clone();

            tokio::spawn(async move {
                let result = llm.query_simple(&prompt, None).await;
                let response = match result {
                    Ok(content) => format!("[TASK_COMPLETE]{}::{}", task_id, content),
                    Err(e) => format!("[TASK_FAILED]{}::{}", task_id, e),
                };
                let _ = tx_clone.send(response).await;
            });
        }
    }

    pub fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Handle background task notifications.
    /// Returns the tab index that was modified, if any.
    pub fn handle_background_notification(&mut self, notification: &str) -> Option<usize> {
        if notification.starts_with("[TASK_COMPLETE]") {
            let parts: Vec<&str> = notification
                .strip_prefix("[TASK_COMPLETE]")
                .map(|s| s.splitn(2, "::").collect())
                .unwrap_or_default();
            if parts.len() == 2 {
                let task_id = parts[0].to_string();
                let result = parts[1].to_string();
                if let Some(task) = self.background_tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = crate::app::TaskStatus::Completed;
                    task.result = Some(result.clone());
                }
                let tab_idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
                self.tabs[tab_idx].messages.push(Message::new(format!(
                    "Task {} completed: {}",
                    task_id, result
                )));
                return Some(tab_idx);
            }
        } else if notification.starts_with("[TASK_FAILED]") {
            let parts: Vec<&str> = notification
                .strip_prefix("[TASK_FAILED]")
                .map(|s| s.splitn(2, "::").collect())
                .unwrap_or_default();
            if parts.len() == 2 {
                let task_id = parts[0].to_string();
                let error = parts[1].to_string();
                if let Some(task) = self.background_tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = crate::app::TaskStatus::Failed;
                    task.result = Some(error.clone());
                }
                let tab_idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
                self.tabs[tab_idx]
                    .messages
                    .push(Message::new(format!("Task {} failed: {}", task_id, error)));
                return Some(tab_idx);
            }
        }
        None
    }

    pub fn submit_message(&mut self) {
        if !self.input.is_empty() {
            let user_msg = self.input.clone();
            let tab_idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
            self.tabs[tab_idx]
                .messages
                .push(Message::new(format!("You: {}", user_msg)));

            // Save to history
            if self.history.last().map(|s| s.as_str()) != Some(&user_msg) {
                self.history.push(user_msg.clone());
                self.save_history();
            }
            self.history_index = None;

            if let Some(tx) = &self.prompt_tx {
                let _ = tx.try_send(user_msg);
            }

            self.input.clear();
        }
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.history_index {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_index = Some(idx);
        self.input = self.history[idx].clone();
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.history_index = None;
                self.input.clear();
            }
            Some(i) => {
                self.history_index = Some(i + 1);
                self.input = self.history[i + 1].clone();
            }
        }
    }

    fn history_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".config/opencrust/history.txt")
    }

    fn load_history(&mut self) {
        let path = Self::history_path();
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            self.history = content.lines().map(|l| l.to_string()).collect();
        }
    }

    pub fn save_history(&self) {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = self.history.join("\n");
        let _ = std::fs::write(path, content);
    }

    pub fn handle_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.input.pop();
    }

    pub fn refresh_sidebar(&mut self) {
        if let Ok(entries) = std::fs::read_dir(".") {
            self.sidebar_items = entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            self.sidebar_items.sort();
        }
    }

    // Vim Mode helper methods
    pub fn move_cursor_left(&mut self) {
        if self.vim_cursor_pos > 0 {
            self.vim_cursor_pos -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let char_count = self.input.chars().count();
        if self.vim_cursor_pos < char_count {
            self.vim_cursor_pos += 1;
        }
    }

    pub fn move_to_next_word(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let mut pos = self.vim_cursor_pos;
        // Skip current word
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }
        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        self.vim_cursor_pos = pos.min(chars.len());
    }

    pub fn move_to_prev_word(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let mut pos = self.vim_cursor_pos;
        // Skip whitespace backwards
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        // Skip current word backwards
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        self.vim_cursor_pos = pos;
    }

    pub fn move_to_line_start(&mut self) {
        self.vim_cursor_pos = 0;
    }

    pub fn move_to_line_end(&mut self) {
        self.vim_cursor_pos = self.input.chars().count();
    }

    pub fn delete_line(&mut self) {
        self.input.clear();
        self.vim_cursor_pos = 0;
    }

    pub fn yank_line(&self, clipboard: &mut crate::clipboard::ClipboardManager) -> bool {
        clipboard.copy(&self.input)
    }
}

/// Simple fuzzy string matching score.
/// Returns 0 if no match, higher values for better matches.
/// Prioritizes consecutive character matches and prefix matches.
fn fuzzy_score(haystack: &str, needle: &str) -> u32 {
    let needle_chars: Vec<char> = needle.chars().collect();
    let needle_len = needle_chars.len();

    if needle_len == 0 || needle_len > haystack.chars().count() {
        return 0;
    }

    // Try to find all needle characters in order in haystack
    let mut score = 0u32;
    let mut needle_idx = 0;
    let mut prev_matched_idx: Option<usize> = None;

    for (i, hc) in haystack.chars().enumerate() {
        if needle_idx < needle_len && hc == needle_chars[needle_idx] {
            // Match found
            score += 1;

            // Bonus for consecutive matches
            if let Some(prev) = prev_matched_idx {
                if i == prev + 1 {
                    score += 2;
                }
            }

            // Bonus for prefix match
            if needle_idx == 0 && i == 0 {
                score += 5;
            }

            // Bonus for match after separator (/, _, -, .)
            if let Some(prev) = prev_matched_idx {
                let between: String = haystack.chars().skip(prev + 1).take(i - prev - 1).collect();
                if between
                    .chars()
                    .any(|c| c == '/' || c == '_' || c == '-' || c == '.')
                {
                    score += 3;
                }
            }

            prev_matched_idx = Some(i);
            needle_idx += 1;
        }
    }

    if needle_idx == needle_len { score } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Mode enum ---

    #[test]
    fn mode_debug_and_clone() {
        let modes = [Mode::Normal, Mode::Insert, Mode::Review, Mode::Servers];
        for m in &modes {
            let _ = format!("{m:?}");
        }
    }

    // --- PlanMode enum ---

    #[test]
    fn plan_mode_default_is_disabled() {
        assert_eq!(PlanMode::default(), PlanMode::Disabled);
    }

    #[test]
    fn plan_mode_partial_eq() {
        assert_eq!(PlanMode::Disabled, PlanMode::Disabled);
        assert_eq!(PlanMode::Planning, PlanMode::Planning);
        assert_ne!(PlanMode::Disabled, PlanMode::Planning);
    }

    // --- Message ---

    #[test]
    fn message_new_sets_content() {
        let msg = Message::new("hello".to_string());
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn message_new_sets_timestamp() {
        let msg = Message::new("test".to_string());
        let now = chrono::Utc::now();
        let diff = now - msg.timestamp;
        assert!(diff.num_seconds() < 5, "timestamp should be recent");
    }

    // --- ChangeStatus ---

    #[test]
    fn change_status_variants() {
        assert_eq!(ChangeStatus::Pending, ChangeStatus::Pending);
        assert_eq!(ChangeStatus::Approved, ChangeStatus::Approved);
        assert_eq!(ChangeStatus::Denied, ChangeStatus::Denied);
        assert_ne!(ChangeStatus::Pending, ChangeStatus::Approved);
    }

    // --- TaskStatus ---

    #[test]
    fn task_status_variants() {
        assert_eq!(TaskStatus::Running, TaskStatus::Running);
        assert_eq!(TaskStatus::Completed, TaskStatus::Completed);
        assert_eq!(TaskStatus::Failed, TaskStatus::Failed);
    }

    // --- ProposedChange ---

    #[test]
    fn proposed_change_defaults_to_pending() {
        let pc = ProposedChange {
            path: "/tmp/f".into(),
            original: "a".into(),
            proposed: "b".into(),
            status: ChangeStatus::Pending,
        };
        assert_eq!(pc.status, ChangeStatus::Pending);
        assert_eq!(pc.path, "/tmp/f");
    }

    // --- BackgroundTask ---

    #[test]
    fn background_task_status_starts_running() {
        let task = BackgroundTask {
            id: "id-1".into(),
            prompt: "hello".into(),
            status: TaskStatus::Running,
            result: None,
            started_at: chrono::Utc::now(),
        };
        assert_eq!(task.status, TaskStatus::Running);
        assert!(task.result.is_none());
    }

    // --- App: mode transitions ---

    #[test]
    fn app_enter_insert_mode() {
        let mut app = app_for_test();
        app.mode = Mode::Normal;
        app.enter_insert_mode();
        assert_eq!(app.mode, Mode::Insert);
    }

    #[test]
    fn app_enter_normal_mode() {
        let mut app = app_for_test();
        app.mode = Mode::Insert;
        app.enter_normal_mode();
        assert_eq!(app.mode, Mode::Normal);
    }

    // --- App: ghost text / prediction ---

    #[test]
    fn should_trigger_prediction_disabled() {
        let app = app_for_test();
        assert!(!app.should_trigger_prediction());
    }

    #[test]
    fn should_trigger_prediction_no_last_time() {
        let mut app = app_for_test();
        app.input_prediction_enabled = true;
        assert!(!app.should_trigger_prediction());
    }

    #[test]
    fn clear_ghost_text_clears_state() {
        let mut app = app_for_test();
        app.ghost_text = Some("test".into());
        app.last_input_time = Some(std::time::Instant::now());
        app.clear_ghost_text();
        assert!(app.ghost_text.is_none());
        assert!(app.last_input_time.is_none());
    }

    // --- App: history navigation ---

    #[test]
    fn history_up_empty_does_nothing() {
        let mut app = app_for_test();
        app.history_up();
        assert!(app.history_index.is_none());
    }

    #[test]
    fn history_up_navigates_entries() {
        let mut app = app_for_test();
        app.history = vec!["first".into(), "second".into()];
        app.history_up();
        assert_eq!(app.history_index, Some(1));
        assert_eq!(app.input, "second");
    }

    #[test]
    fn history_up_stays_at_zero() {
        let mut app = app_for_test();
        app.history = vec!["only".into()];
        app.history_index = Some(0);
        app.history_up();
        assert_eq!(app.history_index, Some(0));
    }

    #[test]
    fn history_down_clears_at_end() {
        let mut app = app_for_test();
        app.history = vec!["a".into()];
        app.history_index = Some(0);
        app.history_down();
        assert!(app.history_index.is_none());
        assert!(app.input.is_empty());
    }

    #[test]
    fn history_down_moves_forward() {
        let mut app = app_for_test();
        app.history = vec!["a".into(), "b".into()];
        app.history_index = Some(0);
        app.history_down();
        assert_eq!(app.history_index, Some(1));
        assert_eq!(app.input, "b");
    }

    #[test]
    fn history_down_none_does_nothing() {
        let mut app = app_for_test();
        app.history_down();
        assert!(app.history_index.is_none());
    }

    // --- App: input manipulation ---

    #[test]
    fn handle_char_appends_to_input() {
        let mut app = app_for_test();
        app.handle_char('x');
        assert_eq!(app.input, "x");
    }

    #[test]
    fn handle_backspace_removes_last_char() {
        let mut app = app_for_test();
        app.input = "ab".into();
        app.handle_backspace();
        assert_eq!(app.input, "a");
    }

    #[test]
    fn handle_backspace_empty_does_nothing() {
        let mut app = app_for_test();
        app.handle_backspace();
        assert!(app.input.is_empty());
    }

    // --- App: vim cursor ---

    #[test]
    fn vim_cursor_left_decrements() {
        let mut app = app_for_test();
        app.vim_cursor_pos = 3;
        app.move_cursor_left();
        assert_eq!(app.vim_cursor_pos, 2);
    }

    #[test]
    fn vim_cursor_left_stays_at_zero() {
        let mut app = app_for_test();
        app.vim_cursor_pos = 0;
        app.move_cursor_left();
        assert_eq!(app.vim_cursor_pos, 0);
    }

    #[test]
    fn vim_cursor_right_increments() {
        let mut app = app_for_test();
        app.input = "abc".into();
        app.vim_cursor_pos = 1;
        app.move_cursor_right();
        assert_eq!(app.vim_cursor_pos, 2);
    }

    #[test]
    fn vim_cursor_right_stays_at_len() {
        let mut app = app_for_test();
        app.input = "ab".into();
        app.vim_cursor_pos = 2;
        app.move_cursor_right();
        assert_eq!(app.vim_cursor_pos, 2);
    }

    #[test]
    fn vim_next_word_skips_whitespace() {
        let mut app = app_for_test();
        app.input = "hello world".into();
        app.vim_cursor_pos = 0;
        app.move_to_next_word();
        assert_eq!(app.vim_cursor_pos, 6);
    }

    #[test]
    fn vim_prev_word_goes_back() {
        let mut app = app_for_test();
        app.input = "hello world".into();
        app.vim_cursor_pos = 7;
        app.move_to_prev_word();
        assert_eq!(app.vim_cursor_pos, 6);
    }

    #[test]
    fn vim_line_start() {
        let mut app = app_for_test();
        app.vim_cursor_pos = 5;
        app.move_to_line_start();
        assert_eq!(app.vim_cursor_pos, 0);
    }

    #[test]
    fn vim_line_end() {
        let mut app = app_for_test();
        app.input = "hello".into();
        app.vim_cursor_pos = 0;
        app.move_to_line_end();
        assert_eq!(app.vim_cursor_pos, 5);
    }

    #[test]
    fn vim_delete_line() {
        let mut app = app_for_test();
        app.input = "hello".into();
        app.vim_cursor_pos = 3;
        app.delete_line();
        assert!(app.input.is_empty());
        assert_eq!(app.vim_cursor_pos, 0);
    }

    // --- App: background notifications ---

    #[test]
    fn handle_bg_completion_updates_task() {
        let mut app = app_for_test();
        let task_id = "bg-001".to_string();
        app.background_tasks.push(BackgroundTask {
            id: task_id.clone(),
            prompt: "hello".into(),
            status: TaskStatus::Running,
            result: None,
            started_at: chrono::Utc::now(),
        });
        let result =
            app.handle_background_notification(&format!("[TASK_COMPLETE]{}::done", task_id));
        assert!(result.is_some());
        assert_eq!(app.background_tasks[0].status, TaskStatus::Completed);
        assert_eq!(app.background_tasks[0].result, Some("done".into()));
    }

    #[test]
    fn handle_bg_failure_updates_task() {
        let mut app = app_for_test();
        let task_id = "bg-002".to_string();
        app.background_tasks.push(BackgroundTask {
            id: task_id.clone(),
            prompt: "hello".into(),
            status: TaskStatus::Running,
            result: None,
            started_at: chrono::Utc::now(),
        });
        let result =
            app.handle_background_notification(&format!("[TASK_FAILED]{}::error msg", task_id));
        assert!(result.is_some());
        assert_eq!(app.background_tasks[0].status, TaskStatus::Failed);
        assert_eq!(app.background_tasks[0].result, Some("error msg".into()));
    }

    #[test]
    fn handle_bg_unknown_notification_returns_none() {
        let mut app = app_for_test();
        let result = app.handle_background_notification("some random text");
        assert!(result.is_none());
    }

    #[test]
    fn handle_bg_malformed_notification_returns_none() {
        let mut app = app_for_test();
        let result = app.handle_background_notification("[TASK_COMPLETE]no_separator");
        assert!(result.is_none());
    }

    #[test]
    fn submit_message_adds_to_tab_and_history() {
        let mut app = app_for_test();
        app.input = "hello".into();
        let initial_count = app.tabs[0].messages.len();
        app.submit_message();
        assert!(app.tabs[0].messages.len() > initial_count);
        assert!(
            app.tabs[0]
                .messages
                .last()
                .unwrap()
                .content
                .contains("hello")
        );
        assert!(app.history.contains(&"hello".into()));
        assert!(app.input.is_empty());
    }

    #[test]
    fn submit_message_empty_does_nothing() {
        let mut app = app_for_test();
        app.history.clear();
        app.submit_message();
        assert!(app.history.is_empty());
        assert!(app.input.is_empty());
    }

    #[test]
    fn submit_message_does_not_duplicate_history() {
        let mut app = app_for_test();
        app.history = vec!["same".into()];
        app.input = "same".into();
        app.submit_message();
        // Should still be just one entry (dedup logic in submit_message)
        assert_eq!(app.history.len(), 1);
    }

    // --- App: tab management ---

    #[test]
    fn tabs_initialized_with_chat_and_tasks() {
        let app = app_for_test();
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.tabs[0].name, "Chat");
        assert_eq!(app.tabs[1].name, "Tasks");
        assert_eq!(app.active_tab, 0);
    }

    #[test]
    fn app_default_mode_is_normal() {
        let app = app_for_test();
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn app_plan_mode_default_disabled() {
        let app = app_for_test();
        assert_eq!(app.plan_mode, PlanMode::Disabled);
    }

    // --- helpers ---

    /// Create an App for testing. Uses real LlmClient with minimal managers.
    /// History is cleared to avoid interference from the on-disk history file.
    fn app_for_test() -> App {
        use std::sync::Arc;
        let (prompt_tx, _) = mpsc::channel(16);
        let (approval_tx, _) = mpsc::channel(16);
        let (bg_tx, _) = mpsc::channel(16);
        let config = Config::default();
        let config_arc = Arc::new(config.clone());
        let llm_client = crate::llm::new_test_client(config_arc).expect("test LlmClient creation");
        let mut app = App::new(config, prompt_tx, approval_tx, bg_tx, llm_client);
        app.history.clear();
        app.history_index = None;
        app
    }
}
