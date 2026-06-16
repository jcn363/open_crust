//! Type definitions for the application state

use chrono;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Review,
    Servers,
    SkillBrowser,   // Ctrl+Shift+K skill browser
    PluginBrowser ,  // Ctrl+P plugin browser
    CommandPalette, // Ctrl+K command palette
    Help          , // ? key help
    McpShowcase   , // Ctrl+M MCP Showcase
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