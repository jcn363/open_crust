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
    pub path: String,
    pub original: String,
    pub proposed: String,
}

pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub input: String,
    pub messages: Vec<String>,
    pub should_quit: bool,
    pub prompt_tx: Option<mpsc::Sender<String>>,
    pub approval_tx: Option<mpsc::Sender<bool>>,
    pub waiting_for_approval: bool,
    pub proposed_changes: Vec<ProposedChange>,
    pub pinned_files: Vec<String>,
    pub llm_client: crate::llm::LlmClient,
    pub mcp_input: String,
}

impl App {
    pub fn new(config: Config, prompt_tx: mpsc::Sender<String>, approval_tx: mpsc::Sender<bool>, llm_client: crate::llm::LlmClient) -> Self {
        Self {
            config,
            mode: Mode::Normal,
            input: String::new(),
            messages: vec![String::from("Welcome to open_crust. Press 'i' to enter insert mode, 's' for servers, 'q' to quit.")],
            should_quit: false,
            prompt_tx: Some(prompt_tx),
            approval_tx: Some(approval_tx),
            waiting_for_approval: false,
            proposed_changes: Vec::new(),
            pinned_files: Vec::new(),
            llm_client,
            mcp_input: String::new(),
        }
    }

    pub fn on_tick(&mut self) {}

    pub fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn submit_message(&mut self) {
        if !self.input.is_empty() {
            let user_msg = self.input.clone();
            self.messages.push(format!("You: {}", user_msg));
            
            if let Some(tx) = &self.prompt_tx {
                let _ = tx.try_send(user_msg);
            }

            self.input.clear();
        }
    }

    pub fn handle_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.input.pop();
    }
}
