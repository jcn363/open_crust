//! Mission Control module for visualizing the orchestrator task DAG
//! Provides TUI (ratatui) for real-time task graph monitoring

mod render;
mod state;
mod types;

pub use state::MissionControlUI;
pub use types::MissionControlAction;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Just verify the module compiles and exports the expected items
        let _ = MissionControlUI::new();
        let _ = MissionControlAction::None;
    }
}
