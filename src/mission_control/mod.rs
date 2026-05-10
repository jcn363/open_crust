//! Mission Control module for visualizing the orchestrator task DAG
//! Provides TUI (ratatui) for real-time task graph monitoring

pub mod tui;
pub use tui::MissionControlAction;
pub use tui::MissionControlUI;
