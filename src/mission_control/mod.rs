
//! Mission Control module for visualizing the orchestrator task DAG
//! Provides TUI (ratatui) for real-time task graph monitoring

pub mod tui;
pub use tui::MissionControlAction;
pub use tui::MissionControlUI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Just verify the module compiles and exports the expected items
        let _ = tui::MissionControlUI::new();
        let _ = tui::MissionControlAction::None;
    }
}
