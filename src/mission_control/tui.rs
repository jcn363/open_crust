//! Mission Control TUI component for visualizing orchestrator task DAG
//! Uses ratatui for terminal rendering

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crate::orchestrator::task::Task;

/// Actions that can be returned by Mission Control TUI
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
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
}

/// Mission Control TUI state
pub struct MissionControlUI {
    /// Snapshot of tasks from orchestrator
    pub tasks: Vec<Task>,
    /// Topological layers: each Vec<usize> = indices into tasks
    pub layers: Vec<Vec<usize>>,
    /// Edge list: (from_task_index, to_task_index)
    pub edges: Vec<(usize, usize)>,
    /// Currently selected task index
    pub selected_index: usize,
    /// Vertical scroll offset for DAG panel
    pub scroll_offset: usize,
    /// Active panel: 0 = DAG, 1 = detail
    pub active_panel: usize,
    /// Dashboard stats
    pub stats: DashboardStats,
}

impl MissionControlUI {
    /// Create a new Mission Control UI with no tasks
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            layers: Vec::new(),
            edges: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            active_panel: 0,
            stats: DashboardStats::default(),
        }
    }

    /// Refresh task snapshot from shared bridge and recompute layout
    pub fn refresh_tasks(&mut self, _bridge: &Option<std::sync::Arc<tokio::sync::RwLock<Vec<Task>>>>) {
        // TODO: Phase 3 - read from bridge and call refresh_layout()
    }

    /// Recompute DAG layout (topological sort, layer assignment, edges)
    pub fn refresh_layout(&mut self) {
        // TODO: Phase 3 - DAG layout engine
    }

    /// Handle key events for navigation
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> MissionControlAction {
        match key {
            crossterm::event::KeyCode::Esc => MissionControlAction::ExitMode,
            _ => MissionControlAction::None,
        }
    }

    /// Render the DAG panel (left side)
    fn render_dag_panel(&mut self, _f: &mut Frame, _area: Rect) {
        // TODO: Phase 4 - DAG rendering with node blocks and unicode edges
    }

    /// Render the detail panel (right side)
    fn render_detail_panel(&mut self, _f: &mut Frame, _area: Rect) {
        // TODO: Phase 5 - task details and dashboard
    }

    /// Main render function
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(60),
                ratatui::layout::Constraint::Percentage(40),
            ])
            .split(area);

        // DAG panel (left 60%)
        let dag_block = Block::default()
            .borders(Borders::ALL)
            .title("Task Graph")
            .style(Style::default().fg(Color::Cyan));
        let dag_inner = dag_block.inner(chunks[0]);
        f.render_widget(dag_block, chunks[0]);

        if self.tasks.is_empty() {
            let empty = Paragraph::new("No active task graph")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(empty, dag_inner);
        } else {
            self.render_dag_panel(f, dag_inner);
        }

        // Detail panel (right 40%)
        let detail_block = Block::default()
            .borders(Borders::ALL)
            .title("Details")
            .style(Style::default().fg(Color::White));
        let detail_inner = detail_block.inner(chunks[1]);
        f.render_widget(detail_block, chunks[1]);

        if self.tasks.is_empty() {
            let empty = Paragraph::new("Select a task to view details")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(empty, detail_inner);
        } else {
            self.render_detail_panel(f, detail_inner);
        }
    }
}

impl Default for MissionControlUI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    fn create_test_ui() -> MissionControlUI {
        MissionControlUI::new()
    }

    #[test]
    fn test_initial_state() {
        let ui = create_test_ui();
        assert!(ui.tasks.is_empty());
        assert!(ui.layers.is_empty());
        assert!(ui.edges.is_empty());
        assert_eq!(ui.selected_index, 0);
        assert_eq!(ui.active_panel, 0);
    }

    #[test]
    fn test_handle_key_esc() {
        let mut ui = create_test_ui();
        let action = ui.handle_key(KeyCode::Esc);
        assert!(matches!(action, MissionControlAction::ExitMode));
    }

    #[test]
    fn test_handle_key_unknown() {
        let mut ui = create_test_ui();
        let action = ui.handle_key(KeyCode::Char('x'));
        assert!(matches!(action, MissionControlAction::None));
    }
}
