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

#[derive(Subcommand, Debug, Clone)]
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
    /// Enterprise compliance packaging (SOC2 reports, evidence, export)
    Compliance {
        #[command(subcommand)]
        cmd: ComplianceCommands,
    },
    /// Background agent management and dashboard
    /// Agent dashboard TUI
    Background {
        #[command(subcommand)]
        cmd: BackgroundCommands,
    },
    /// Plugin/extension management
    Plugin {
        #[command(subcommand)]
        cmd: PluginCommands,
    },
    /// Multi-repository management
    Repo {
        #[command(subcommand)]
        cmd: RepoCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
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

#[derive(Subcommand, Debug, Clone)]
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

#[derive(Subcommand, Debug, Clone)]
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
    /// Create a checkpoint (snapshot) of a session for rollback
    Checkpoint {
        /// Session ID to checkpoint
        id: String,
        /// Optional name for the checkpoint (defaults to timestamp)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List all checkpoints for a session
    CheckpointList {
        /// Session ID
        id: String,
    },
    /// Restore a session from a checkpoint
    CheckpointRestore {
        /// Session ID
        id: String,
        /// Checkpoint name to restore
        name: String,
    },
    /// Delete a checkpoint
    CheckpointDelete {
        /// Session ID
        id: String,
        /// Checkpoint name to delete
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SkillsCommands {
    /// List all skills with their status
    List,
    /// Activate a skill
    Activate { name: String },
    /// Deactivate a skill
    Deactivate { name: String },
}

#[derive(Subcommand, Debug, Clone)]
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
    /// Run enterprise compliance policy check
    Policy {
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// Verify an existing evidence package integrity
    Verify {
        /// Path to the evidence package directory
        path: String,
    },
    /// Run a full compliance check (policy + evidence + report)
    Check {
        #[arg(long, default_value = ".")]
        output_dir: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum BackgroundCommands {
    /// List all background agents
    List,
    /// Show details for a specific agent
    Show {
        /// Agent UUID
        id: String,
    },
    /// Start a new background agent
    Start {
        /// Human-readable name for the agent
        name: String,
        /// Prompt for the agent to execute
        prompt: String,
    },
    /// Cancel a running agent
    Cancel {
        /// Agent UUID to cancel
        id: String,
    },
    /// Show agent statistics
    Stats,
    /// Tail logs from a specific agent
    Logs {
        /// Agent UUID
        id: String,
        #[arg(long, default_value_t = 50)]
        lines: usize,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum PluginCommands {
    /// List all discovered plugins
    List,
    /// Show details for a specific plugin
    Show {
        /// Plugin name
        name: String,
    },
    /// Install a plugin from a directory or manifest path
    Install {
        /// Path to plugin directory or plugin.json
        path: String,
    },
    /// Remove an installed plugin
    Remove {
        /// Plugin name to remove
        name: String,
    },
    /// Enable a plugin
    Enable {
        /// Plugin name to enable
        name: String,
    },
    /// Disable a plugin
    Disable {
        /// Plugin name to disable
        name: String,
    },
    /// Show plugin statistics
    Stats,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RepoCommands {
    /// List all registered repositories
    List,
    /// Show details for a specific repository
    Show {
        /// Repository name
        name: String,
    },
    /// Register a new repository
    Add {
        /// Short name/alias for the repo
        name: String,
        /// Local path to the repository
        path: String,
        /// Optional tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Remove a registered repository
    Remove {
        /// Repository name to remove
        name: String,
    },
    /// Show repository statistics
    Stats,
    /// Run a git command across all repos
    Git {
        /// Git arguments (e.g., "status --short")
        args: Vec<String>,
    },
    /// Search file names across all repos
    Search {
        /// File name pattern to search for
        pattern: String,
    },
    /// Refresh all repository metadata
    Refresh,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ComplianceCommands {
    /// Generate SOC 2 Type II compliance report from evidence packages
    Generate {
        /// Output directory for the report
        #[arg(long, default_value = ".")]
        output_dir: String,
        /// Report format: text, json, html, soc2
        #[arg(long, default_value = "soc2")]
        format: String,
        /// Include evidence package in output
        #[arg(long)]
        include_evidence: bool,
        /// Compliance framework: soc2, hipaa, sox, pci-dss, iso27001
        #[arg(long, default_value = "soc2")]
        framework: String,
    },
    /// Export audit logs in multiple formats (CSV, JSON, Syslog)
    Export {
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
        /// Filter by tool/action name
        #[arg(long)]
        action: Option<String>,
        /// Filter by approval status (approved/denied)
        #[arg(long)]
        status: Option<String>,
        /// Export format: csv, json, syslog
        #[arg(long, default_value = "csv")]
        format: String,
        /// Output file path (stdout if not specified)
        #[arg(long)]
        output: Option<String>,
        /// Syslog server address (for syslog format)
        #[arg(long)]
        syslog_server: Option<String>,
        /// Syslog facility (for syslog format)
        #[arg(long, default_value = "local0")]
        syslog_facility: String,
    },
    /// Manage role-based permission templates
    Permissions {
        #[command(subcommand)]
        cmd: PermissionCommands,
    },
    /// Build evidence package with SHA256 manifest
    Evidence {
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// Verify an existing evidence package integrity
    Verify {
        /// Path to the evidence package directory
        path: String,
    },
    /// Run a full compliance check (policy + evidence + report)
    Check {
        #[arg(long, default_value = ".")]
        output_dir: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum PermissionCommands {
    /// List available role templates
    List,
    /// Show details for a specific role template
    Show {
        /// Role name: admin, developer, reviewer
        role: String,
    },
    /// Export role template as JSON
    Export {
        /// Role name: admin, developer, reviewer
        role: String,
        /// Output file path (stdout if not specified)
        #[arg(long)]
        output: Option<String>,
    },
    /// Apply a role template to current configuration
    Apply {
        /// Role name: admin, developer, reviewer
        role: String,
    },
}
