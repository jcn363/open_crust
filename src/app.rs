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
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                self.history = content.lines().map(|l| l.to_string()).collect();
            }
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
