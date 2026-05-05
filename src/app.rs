use crate::config::Config;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Review,
    Servers,
}

#[derive(Clone, Debug)]
pub struct ProposedChange {
    pub original: String,
    pub proposed: String,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub name: String,
    pub messages: Vec<String>,
}

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
    pub sidebar_items: Vec<String>,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    // MCP Browser state
    pub mcp_browser_items: Vec<(String, String, Vec<String>)>, // (name, description, command)
    pub mcp_browser_selected: usize,
    pub mcp_browser_scroll: usize,
}

impl App {
    pub fn new(config: Config, prompt_tx: mpsc::Sender<String>, approval_tx: mpsc::Sender<bool>, llm_client: crate::llm::LlmClient) -> Self {
        let chat_tab = Tab {
            name: "Chat".to_string(),
            messages: vec![String::from("Welcome to open_crust. Press 'i' to enter insert mode, 's' for servers, 'q' to quit.")],
        };
        let tasks_tab = Tab {
            name: "Tasks".to_string(),
            messages: vec![String::from("No tasks yet.")],
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
            history: Vec::new(),
            history_index: None,
            // MCP Browser initialization
            mcp_browser_items: vec![
                ("github".to_string(), "GitHub integration".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-github".to_string()]),
                ("slack".to_string(), "Slack integration".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-slack".to_string()]),
                ("filesystem".to_string(), "File system access".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()]),
                ("postgres".to_string(), "PostgreSQL database".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-postgres".to_string()]),
                ("google-drive".to_string(), "Google Drive".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-google-drive".to_string()]),
                ("git".to_string(), "Git repository tools".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-git".to_string()]),
                ("sqlite".to_string(), "SQLite database".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-sqlite".to_string()]),
                ("brave-search".to_string(), "Brave search API".to_string(), vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()]),
            ],
            mcp_browser_selected: 0,
            mcp_browser_scroll: 0,
        };
        app.load_history();
        app
    }

    pub fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn submit_message(&mut self) {
        if !self.input.is_empty() {
            let user_msg = self.input.clone();
            self.tabs[self.active_tab].messages.push(format!("You: {}", user_msg));

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
        if self.history.is_empty() { return; }
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
            .join(".config/open_crust/history.txt")
    }

    fn load_history(&mut self) {
        let path = Self::history_path();
        if path.exists() && let Ok(content) = std::fs::read_to_string(path) {
            self.history = content.lines().map(|l| l.to_string()).collect();
        }
    }

    fn save_history(&self) {
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
            self.sidebar_items = entries.flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            self.sidebar_items.sort();
        }
    }
}
