//! Mission Control data types

use super::spaces::Space;

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
    // Space management
    SelectSpace(usize),
    CreateSpace(String),
    DeleteSpace(String),
    ArchiveSpace(String),
    // Agent dashboard
    SelectAgent(usize),
    PauseAgent(String),
    ResumeAgent(String),
    CancelAgent(String),
    // View modes
    ToggleSpacePanel,
    ToggleAgentDashboard,
}

/// Agent panel state
pub struct AgentPanel {
    pub agents: Vec<crate::background_agents::BackgroundAgent>,
    pub selected: usize,
    pub show_logs: bool,
}

impl AgentPanel {
    /// Create a new agent panel
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            selected: 0,
            show_logs: false,
        }
    }
}

impl Default for AgentPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Space panel state
pub struct SpacePanel {
    pub spaces: Vec<Space>,
    pub selected: usize,
    pub show_archived: bool,
}

impl SpacePanel {
    /// Create a new space panel
    pub fn new() -> Self {
        Self {
            spaces: Vec::new(),
            selected: 0,
            show_archived: false,
        }
    }

    /// Get visible spaces (filtered by archive status)
    pub fn visible_spaces(&self) -> Vec<&Space> {
        self.spaces
            .iter()
            .filter(|s| self.show_archived || !s.archived)
            .collect()
    }
}

impl Default for SpacePanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Dashboard statistics snapshot
#[derive(Debug, Clone, Default)]
pub struct DashboardStats {
    pub total: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub active_agents: usize,
    pub idle_agents: usize,
    pub failed_agents: usize,
    pub success_rate: f32,
}

/// Node position in the rendered DAG layout
#[derive(Debug, Clone, Default)]
pub(crate) struct NodePosition {
    pub x: u16,
    pub y: u16,
}

/// View mode for Mission Control
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ViewMode {
    /// DAG view (default)
    #[default]
    Dag,
    /// Space panel view
    Spaces,
    /// Agent dashboard view
    Dashboard,
    /// Combined view with all panels
    Combined,
}
