//! Application state and TUI orchestration
//!
//! Central state management for the terminal UI: holds all application
//! state (chat messages, tabs, input buffer, mode, sidebar, background
//! tasks). Handles event dispatch between UI layers, clipboard integration,
//! message history, and cursor movement within the input prompt.

use crate::config::Config;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Review,
    Servers,
    SkillBrowser,   // Ctrl+Shift+K skill browser
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
    // Plan review index (for diff viewer)
    pub plan_review_index: usize, // Which file in the plan being reviewed
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
                "Welcome to opencrust. Press 'i' to enter insert mode, 's' for servers, 'Ctrl+Q' to quit.",
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
            // Plan review index (for diff viewer)
            plan_review_index: 0,
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
        };
        app.load_history();
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
        if self.vim_cursor_pos < self.input.len() {
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
        self.vim_cursor_pos = self.input.len();
    }

    pub fn delete_line(&mut self) {
        self.input.clear();
        self.vim_cursor_pos = 0;
    }

    pub fn yank_line(&self, clipboard: &mut crate::clipboard::ClipboardManager) -> bool {
        clipboard.copy(&self.input)
    }
}
