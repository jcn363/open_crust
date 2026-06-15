//! Mission Control data types

/// Actions that can be returned by Mission Control TUI
#[derive(Debug, PartialEq)]
pub enum MissionControlAction {
    None,
    SelectTask(usize),
    TogglePanel,
    ExitMode,
}

/// Dashboard statistics snapshot
#[derive(Debug, Clone, Default)]
pub struct DashboardStats {
    pub total: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
}

/// Node position in the rendered DAG layout
#[derive(Debug, Clone, Default)]
pub(crate) struct NodePosition {
    pub x: u16,
    pub y: u16,
}
