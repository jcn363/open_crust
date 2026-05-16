//! CLI argument parsing and subcommand dispatch types
//!
//! Defines all CLI argument structures using clap derive macros.
//! Extracted from `main.rs` to keep the entry point focused on startup logic.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run with multiple agents in parallel (format: provider:model, e.g., ollama:llama3)
    #[arg(long, value_name = "AGENT", num_args = 0..)]
    pub agents: Vec<String>,

    /// Prompt to send to multiple agents (use with --agent)
    #[arg(long, value_name = "PROMPT")]
    pub multi_prompt: Option<String>,

    /// Run in headless mode with a single prompt (no TUI)
    #[arg(short = 'p', long, value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// Read prompt from file (use with --prompt)
    #[arg(short = 'f', long, value_name = "FILE")]
    pub file: Option<String>,

    /// Set working directory for headless mode
    #[arg(long, value_name = "DIR")]
    pub project: Option<String>,

    /// Override provider for this invocation (ollama, openrouter, openai, gemini, mistral, anthropic)
    #[arg(long, value_name = "PROVIDER")]
    pub provider: Option<String>,

    /// Override model for this invocation (default depends on provider; for openrouter default is openrouter/free, no API key required)
    #[arg(long, value_name = "MODEL")]
    pub model: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start in ACP mode (JSON-RPC over stdio)
    Acp,
    /// Run a single command and exit
    Run { command: String },
    /// MCP server management
    Mcp {
        #[command(subcommand)]
        cmd: McpCommands,
    },
    /// Desktop integration features (file picker, notifications)
    Desktop {
        #[command(subcommand)]
        cmd: DesktopCommands,
    },
    /// Session management
    Session {
        #[command(subcommand)]
        cmd: SessionCommands,
    },
    /// Skill management
    Skills {
        #[command(subcommand)]
        cmd: SkillsCommands,
    },
    /// Audit log management and compliance
    Audit {
        #[command(subcommand)]
        cmd: AuditCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// List available MCP servers
    List,
    /// Install an MCP server by name
    Install { server: String },
    /// Launch MCP Showcase TUI browser (opens interactive TUI)
    Browse,
    /// Print MCP Showcase info to terminal (table of configured servers)
    Showcase,
    /// Test an MCP tool by calling it with arguments
    Test {
        /// Server name (e.g., "weather", "github")
        server: String,
        /// Tool name (e.g., "get_alerts", "get_forecast")
        tool: String,
        /// JSON arguments for the tool (e.g., '{"state": "CA"}')
        args: Option<String>,
    },
    /// List all tools across all MCP servers
    Tools,
}

#[derive(Subcommand, Debug)]
pub enum DesktopCommands {
    /// Open file picker dialog
    FilePicker {
        /// Mode: open, open-multiple, save, directory
        #[arg(short, long, default_value = "open")]
        mode: String,
        /// Initial directory
        #[arg(short, long)]
        dir: Option<String>,
        /// Window title
        #[arg(short, long)]
        title: Option<String>,
    },
    /// Send a desktop notification
    Notify {
        /// Notification title
        #[arg(short, long)]
        title: String,
        /// Notification body
        #[arg(short, long)]
        body: String,
        /// Urgency: low, normal, critical
        #[arg(short, long, default_value = "normal")]
        urgency: String,
    },
    /// Detect desktop environment
    Detect,
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// List all sessions
    List,
    /// Show a specific session
    Show { id: String },
    /// Delete a session
    Delete { id: String },
    /// Save current session (requires messages JSON)
    Save {
        id: String,
        #[arg(short, long)]
        messages: String,
    },
    /// Fork a session to experiment with different approaches
    Fork {
        id: String,
        #[arg(short, long)]
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SkillsCommands {
    /// List all skills with their status
    List,
    /// Activate a skill
    Activate { name: String },
    /// Deactivate a skill
    Deactivate { name: String },
}

#[derive(Subcommand, Debug)]
pub enum AuditCommands {
    /// Export audit logs to CSV or JSON
    Export {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        action: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value = "csv")]
        format: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// Query audit logs and display as table
    Query {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        action: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Build evidence package with SHA256 manifest
    Evidence {
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// Generate compliance report
    Report {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },
}
