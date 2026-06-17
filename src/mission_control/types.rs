//! Mission Control data types

/// Width of each node block in the DAG layout
pub(crate) const NODE_WIDTH: u16 = 22;
/// Height of each node block in the DAG layout
pub(crate) const NODE_HEIGHT: u16 = 3;
/// Horizontal gap between layers
pub(crate) const H_GAP: u16 = 4;
/// Vertical gap between nodes in the same layer
pub(crate) const V_GAP: u16 = 1;

/// Actions that can be returned by Mission Control TUI
#[derive(Debug, PartialEq)]
pub enum MissionControlAction {
    None,
    SelectTask(usize),
    TogglePanel,
    ExitMode,
}

/// Actions for agent panel
#[allow(dead_code)] // wired for agent dashboard TUI
#[derive(Debug, PartialEq)]
pub enum AgentAction {
    None,
    StartAgent(String),
    StopAgent(usize),
    ViewAgentLogs(usize),
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

/// Agent panel state
#[allow(dead_code)] // wired for agent dashboard TUI
pub struct AgentPanel {
    pub agents: Vec<crate::background_agents::BackgroundAgent>,
    pub selected: usize,
    pub show_logs: bool,
}
