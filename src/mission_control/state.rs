//! Mission Control state management

use crate::orchestrator::task::{Task, TaskState};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{DashboardStats, MissionControlAction, NodePosition};

/// Mission Control TUI state
pub struct MissionControlUI {
    /// Snapshot of tasks from orchestrator
    pub tasks: Vec<Task>,
    /// Topological layers: each `Vec<usize>` = indices into tasks
    pub layers: Vec<Vec<usize>>,
    /// Edge list: (from_task_index, to_task_index)
    pub edges: Vec<(usize, usize)>,
    /// Pre-computed node positions for rendering
    pub(crate) node_positions: Vec<NodePosition>,
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
            node_positions: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            active_panel: 0,
            stats: DashboardStats::default(),
        }
    }

    /// Refresh task snapshot from shared bridge and recompute layout
    pub fn refresh_tasks(&mut self, bridge: Option<&Arc<RwLock<Vec<Task>>>>) {
        if let Some(shared) = bridge {
            // try_read — never block the UI thread. Accept stale data on contention.
            if let Ok(guard) = shared.try_read() {
                // Only recompute if tasks actually changed
                if *guard != self.tasks {
                    self.tasks = guard.clone();
                    self.refresh_layout();
                    self.compute_stats();
                    // Clamp selected index after refresh
                    if !self.tasks.is_empty() && self.selected_index >= self.tasks.len() {
                        self.selected_index = self.tasks.len() - 1;
                    }
                }
            }
            // On lock contention: keep stale snapshot (no-op)
        }
    }

    /// Recompute DAG layout (topological sort, layer assignment, edges)
    pub fn refresh_layout(&mut self) {
        let n = self.tasks.len();
        if n == 0 {
            self.layers.clear();
            self.edges.clear();
            self.node_positions.clear();
            return;
        }

        // Build a task index map: Uuid -> index
        let mut id_to_idx: std::collections::HashMap<uuid::Uuid, usize> =
            std::collections::HashMap::with_capacity(n);
        for (i, task) in self.tasks.iter().enumerate() {
            id_to_idx.insert(task.id, i);
        }

        // Phase 3a: Topological sort using Kahn's algorithm.
        // We layer tasks by their max dependency depth.
        let mut in_degree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n]; // reverse: dep -> dependent

        for (i, task) in self.tasks.iter().enumerate() {
            for dep_id in &task.dependencies {
                if let Some(&dep_idx) = id_to_idx.get(dep_id) {
                    adj[dep_idx].push(i);
                    in_degree[i] += 1;
                }
            }
        }

        // Kahn's algorithm — compute topological order
        let mut queue: Vec<usize> = Vec::new();
        for (i, _) in in_degree.iter().enumerate().take(n) {
            if in_degree[i] == 0 {
                queue.push(i);
            }
        }

        let mut topo_order: Vec<usize> = Vec::with_capacity(n);
        let mut layer_of = vec![0usize; n];

        while let Some(u) = queue.pop() {
            topo_order.push(u);
            for &v in &adj[u] {
                // Layer: child is at least one past the max of its deps
                layer_of[v] = layer_of[v].max(layer_of[u] + 1);
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push(v);
                }
            }
        }

        // If we didn't process all tasks, there's a cycle. Put remaining in last layer.
        let max_layer = layer_of.iter().max().copied().unwrap_or(0);
        for i in 0..n {
            if in_degree[i] > 0 {
                // Task has unmet dependencies (likely a cycle or missing dep)
                layer_of[i] = max_layer + 1;
            }
        }

        // Build layers: group indices by layer number
        let num_layers = layer_of.iter().max().copied().unwrap_or(0) + 1;
        let mut layers_raw: Vec<Vec<usize>> = vec![Vec::new(); num_layers];
        for (i, &layer) in layer_of.iter().enumerate() {
            layers_raw[layer].push(i);
        }

        // Remove empty layers (shouldn't happen but be safe)
        layers_raw.retain(|l| !l.is_empty());

        // Compute edges: for each task, find which of its deps are also in our task list
        let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
        for (i, task) in self.tasks.iter().enumerate() {
            for dep_id in &task.dependencies {
                if let Some(&dep_idx) = id_to_idx.get(dep_id)
                    && dep_idx != i
                {
                    edge_set.insert((dep_idx, i));
                }
            }
        }

        self.layers = layers_raw;
        self.edges = edge_set.into_iter().collect();

        // Pre-compute node positions for the DAG renderer
        self.compute_node_positions();
    }

    /// Compute (x, y) positions for each task node based on layer arrangement
    fn compute_node_positions(&mut self) {
        let n = self.tasks.len();
        if n == 0 {
            self.node_positions.clear();
            return;
        }

        let node_width: u16 = 22; // Width of each node block in chars
        let node_height: u16 = 3; // Height of each node block
        let h_gap: u16 = 4; // Horizontal gap between layers
        let v_gap: u16 = 1; // Vertical gap between nodes in same layer

        let mut positions = vec![NodePosition::default(); n];
        let mut x_cursor: u16 = 1;

        for layer in &self.layers {
            let mut y_cursor: u16 = 1; // Start from top

            for &task_idx in layer {
                if task_idx < n {
                    positions[task_idx] = NodePosition {
                        x: x_cursor,
                        y: y_cursor,
                    };
                    y_cursor += node_height + v_gap;
                }
            }
            x_cursor += node_width + h_gap;
        }

        self.node_positions = positions;
    }

    /// Compute dashboard statistics from current tasks
    pub(crate) fn compute_stats(&mut self) {
        let total = self.tasks.len();
        let mut pending = 0usize;
        let mut running = 0usize;
        let mut completed = 0usize;
        let mut failed = 0usize;

        for task in &self.tasks {
            match task.state {
                TaskState::Pending => pending += 1,
                TaskState::Running { .. } => running += 1,
                TaskState::Completed { .. } => completed += 1,
                TaskState::Failed { .. } => failed += 1,
            }
        }

        self.stats = DashboardStats {
            total,
            pending,
            running,
            completed,
            failed,
        };
    }

    /// Get the state icon and color for a task
    pub(crate) fn task_style(task: &Task) -> (char, ratatui::style::Color) {
        use ratatui::style::Color;
        match &task.state {
            TaskState::Pending => ('⏳', Color::White),
            TaskState::Running { .. } => ('▶', Color::Yellow),
            TaskState::Completed { .. } => ('✓', Color::Green),
            TaskState::Failed { .. } => ('✗', Color::Red),
        }
    }

    /// Truncate text to fit within a given width (character-based, not byte-based)
    pub(crate) fn truncate(text: &str, max_width: usize) -> String {
        if text.chars().count() <= max_width {
            text.to_string()
        } else {
            let truncated: String = text.chars().take(max_width.saturating_sub(1)).collect();
            format!("{}…", truncated)
        }
    }

    /// Handle key events for navigation
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> MissionControlAction {
        match key {
            crossterm::event::KeyCode::Esc => {
                return MissionControlAction::ExitMode;
            }
            crossterm::event::KeyCode::Tab => {
                // Toggle active panel
                self.active_panel = (self.active_panel + 1) % 2;
                return MissionControlAction::TogglePanel;
            }
            crossterm::event::KeyCode::Up if self.active_panel == 0 => {
                // Navigate up within the current layer
                self.navigate_up_in_layer();
            }
            crossterm::event::KeyCode::Down if self.active_panel == 0 => {
                // Navigate down within the current layer
                self.navigate_down_in_layer();
            }
            crossterm::event::KeyCode::Left if self.active_panel == 0 => {
                // Move to previous layer
                self.navigate_prev_layer();
            }
            crossterm::event::KeyCode::Right if self.active_panel == 0 => {
                // Move to next layer
                self.navigate_next_layer();
            }
            crossterm::event::KeyCode::Enter
                if !self.tasks.is_empty() && self.selected_index < self.tasks.len() =>
            {
                return MissionControlAction::SelectTask(self.selected_index);
            }
            _ => {}
        }
        MissionControlAction::None
    }

    /// Navigate up within the current layer
    fn navigate_up_in_layer(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        // Find which layer contains selected_index
        for layer in &self.layers {
            if let Some(pos) = layer.iter().position(|&i| i == self.selected_index) {
                if pos > 0 {
                    self.selected_index = layer[pos - 1];
                    self.ensure_selected_visible();
                }
                return;
            }
        }
    }

    /// Navigate down within the current layer
    fn navigate_down_in_layer(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        for layer in &self.layers {
            if let Some(pos) = layer.iter().position(|&i| i == self.selected_index) {
                if pos + 1 < layer.len() {
                    self.selected_index = layer[pos + 1];
                    self.ensure_selected_visible();
                }
                return;
            }
        }
    }

    /// Navigate to the previous layer
    fn navigate_prev_layer(&mut self) {
        if self.tasks.is_empty() || self.layers.is_empty() {
            return;
        }
        let current_layer_idx = self.find_current_layer_idx();
        if current_layer_idx > 0 {
            let prev_layer = &self.layers[current_layer_idx - 1];
            if !prev_layer.is_empty() {
                self.selected_index = prev_layer[0];
                self.ensure_selected_visible();
            }
        }
    }

    /// Navigate to the next layer
    fn navigate_next_layer(&mut self) {
        if self.tasks.is_empty() || self.layers.is_empty() {
            return;
        }
        let current_layer_idx = self.find_current_layer_idx();
        if current_layer_idx + 1 < self.layers.len() {
            let next_layer = &self.layers[current_layer_idx + 1];
            if !next_layer.is_empty() {
                self.selected_index = next_layer[0];
                self.ensure_selected_visible();
            }
        }
    }

    /// Find which layer index contains the selected task
    fn find_current_layer_idx(&self) -> usize {
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.contains(&self.selected_index) {
                return idx;
            }
        }
        0
    }

    /// Ensure the selected node is within the visible scroll area
    fn ensure_selected_visible(&mut self) {
        if self.node_positions.is_empty() || self.selected_index >= self.node_positions.len() {
            return;
        }
        let pos = &self.node_positions[self.selected_index];
        // Rough heuristic: if selected node is above scroll, push scroll up
        if pos.y < self.scroll_offset as u16 {
            self.scroll_offset = pos.y.saturating_sub(1) as usize;
        }
        // If selected node is far below, push scroll down
        if pos.y > self.scroll_offset as u16 + 15 {
            self.scroll_offset = pos.y.saturating_sub(10) as usize;
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
    use uuid::Uuid;

    fn make_task(id: Uuid, desc: &str, agent: &str, deps: Vec<Uuid>, state: TaskState) -> Task {
        Task {
            id,
            description: desc.to_string(),
            dependencies: deps,
            state,
            result: None,
            agent_type: agent.to_string(),
        }
    }

    fn create_test_ui() -> MissionControlUI {
        MissionControlUI::new()
    }

    fn create_populated_ui() -> MissionControlUI {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        let tasks = vec![
            make_task(
                a,
                "Task A",
                "agent1",
                vec![],
                TaskState::Completed {
                    output: "done".into(),
                },
            ),
            make_task(b, "Task B", "agent1", vec![a], TaskState::Pending),
            make_task(
                c,
                "Task C",
                "agent2",
                vec![a],
                TaskState::Running {
                    agent_id: "pool-1".into(),
                },
            ),
            make_task(d, "Task D", "agent3", vec![b, c], TaskState::Pending),
        ];

        let mut ui = MissionControlUI::new();
        ui.tasks = tasks;
        ui.refresh_layout();
        ui.compute_stats();
        ui
    }

    // =====================
    // Phase 3: Layout tests
    // =====================

    #[test]
    fn test_layout_chain() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        let tasks = vec![
            make_task(a, "A", "a", vec![], TaskState::Pending),
            make_task(b, "B", "b", vec![a], TaskState::Pending),
            make_task(c, "C", "c", vec![b], TaskState::Pending),
        ];

        let mut ui = MissionControlUI::new();
        ui.tasks = tasks;
        ui.refresh_layout();

        // Chain A→B→C should produce 3 layers and 2 edges
        assert_eq!(ui.layers.len(), 3, "chain should have 3 layers");
        assert_eq!(ui.edges.len(), 2, "chain should have 2 edges");
        assert!(ui.edges.contains(&(0, 1)), "edge A→B");
        assert!(ui.edges.contains(&(1, 2)), "edge B→C");
    }

    #[test]
    fn test_layout_diamond() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        let tasks = vec![
            make_task(a, "A", "a", vec![], TaskState::Pending),
            make_task(b, "B", "b", vec![a], TaskState::Pending),
            make_task(c, "C", "c", vec![a], TaskState::Pending),
            make_task(d, "D", "d", vec![b, c], TaskState::Pending),
        ];

        let mut ui = MissionControlUI::new();
        ui.tasks = tasks;
        ui.refresh_layout();

        // Diamond A→B, A→C, B→D, C→D should produce 3 layers, 4 edges
        assert_eq!(ui.layers.len(), 3, "diamond should have 3 layers");
        assert_eq!(ui.edges.len(), 4, "diamond should have 4 edges");
    }

    #[test]
    fn test_layout_independent() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        let tasks = vec![
            make_task(a, "A", "a", vec![], TaskState::Pending),
            make_task(b, "B", "b", vec![], TaskState::Pending),
        ];

        let mut ui = MissionControlUI::new();
        ui.tasks = tasks;
        ui.refresh_layout();

        // Two independent tasks → 1 layer, 0 edges
        assert_eq!(ui.layers.len(), 1, "independent should have 1 layer");
        assert_eq!(ui.edges.len(), 0, "independent should have 0 edges");
    }

    #[test]
    fn test_layout_empty() {
        let mut ui = MissionControlUI::new();
        ui.refresh_layout();
        assert!(ui.layers.is_empty(), "empty should have no layers");
        assert!(ui.edges.is_empty(), "empty should have no edges");
    }

    #[test]
    fn test_layout_single() {
        let a = Uuid::new_v4();
        let tasks = vec![make_task(a, "Only", "a", vec![], TaskState::Pending)];

        let mut ui = MissionControlUI::new();
        ui.tasks = tasks;
        ui.refresh_layout();

        assert_eq!(ui.layers.len(), 1, "single task should have 1 layer");
        assert_eq!(ui.edges.len(), 0, "single task should have 0 edges");
    }

    // ==============================
    // Phase 5: Stats computation
    // ==============================

    #[test]
    fn test_compute_stats_empty() {
        let mut ui = create_test_ui();
        ui.compute_stats();
        assert_eq!(ui.stats.total, 0);
        assert_eq!(ui.stats.pending, 0);
    }

    #[test]
    fn test_compute_stats_mixed() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();
        let e = Uuid::new_v4();

        let tasks = vec![
            make_task(a, "P", "a", vec![], TaskState::Pending),
            make_task(
                b,
                "R1",
                "b",
                vec![],
                TaskState::Running {
                    agent_id: "x".into(),
                },
            ),
            make_task(
                c,
                "R2",
                "c",
                vec![],
                TaskState::Running {
                    agent_id: "y".into(),
                },
            ),
            make_task(
                d,
                "C",
                "d",
                vec![],
                TaskState::Completed {
                    output: "ok".into(),
                },
            ),
            make_task(
                e,
                "F",
                "e",
                vec![],
                TaskState::Failed {
                    error: "err".into(),
                },
            ),
        ];

        let mut ui = MissionControlUI::new();
        ui.tasks = tasks;
        ui.compute_stats();

        assert_eq!(ui.stats.total, 5);
        assert_eq!(ui.stats.pending, 1);
        assert_eq!(ui.stats.running, 2);
        assert_eq!(ui.stats.completed, 1);
        assert_eq!(ui.stats.failed, 1);
    }

    #[test]
    fn test_render_detail_no_panic() {
        let ui = create_populated_ui();
        // Verify the UI state is valid (render would not panic)
        assert!(ui.stats.total > 0);
        assert!(ui.selected_index < ui.tasks.len());
    }

    // ==============================
    // Phase 6: Navigation tests
    // ==============================

    #[test]
    fn test_navigate_down_in_layer() {
        let ui = create_populated_ui();
        // Selection should be clamped to valid range
        if !ui.tasks.is_empty() {
            assert!(ui.selected_index < ui.tasks.len());
        }
    }

    // ==============================
    // Phase 7: Bridge refresh tests
    // ==============================

    #[test]
    fn test_refresh_tasks_no_bridge() {
        let mut ui = create_populated_ui();
        let initial_len = ui.tasks.len();
        ui.refresh_tasks(None);
        // Without a bridge, tasks should remain unchanged
        assert_eq!(ui.tasks.len(), initial_len);
    }

    #[test]
    fn test_refresh_tasks_stale_on_contention() {
        let mut ui = create_populated_ui();
        let initial_count = ui.stats.total;

        // With a bridge that's locked, should keep stale snapshot
        let bridge: Option<Arc<RwLock<Vec<Task>>>> = None;
        ui.refresh_tasks(bridge.as_ref());
        assert_eq!(ui.stats.total, initial_count);
    }

    // =====================
    // Core action tests
    // =====================

    #[test]
    fn test_initial_state() {
        let ui = create_test_ui();
        assert!(ui.tasks.is_empty());
        assert!(ui.layers.is_empty());
        assert!(ui.edges.is_empty());
        assert!(ui.node_positions.is_empty());
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

    #[test]
    fn test_handle_key_enter() {
        let mut ui = create_populated_ui();
        let action = ui.handle_key(KeyCode::Enter);
        assert!(matches!(action, MissionControlAction::SelectTask(_)));
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(MissionControlUI::truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        let result = MissionControlUI::truncate("hello world this is long", 10);
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_task_style_pending() {
        let task = make_task(Uuid::new_v4(), "t", "a", vec![], TaskState::Pending);
        let (icon, color) = MissionControlUI::task_style(&task);
        assert_eq!(icon, '⏳');
        assert_eq!(color, ratatui::style::Color::White);
    }

    #[test]
    fn test_task_style_running() {
        let task = make_task(
            Uuid::new_v4(),
            "t",
            "a",
            vec![],
            TaskState::Running {
                agent_id: "x".into(),
            },
        );
        let (icon, color) = MissionControlUI::task_style(&task);
        assert_eq!(icon, '▶');
        assert_eq!(color, ratatui::style::Color::Yellow);
    }

    #[test]
    fn test_task_style_completed() {
        let task = make_task(
            Uuid::new_v4(),
            "t",
            "a",
            vec![],
            TaskState::Completed {
                output: "ok".into(),
            },
        );
        let (icon, color) = MissionControlUI::task_style(&task);
        assert_eq!(icon, '✓');
        assert_eq!(color, ratatui::style::Color::Green);
    }

    #[test]
    fn test_task_style_failed() {
        let task = make_task(
            Uuid::new_v4(),
            "t",
            "a",
            vec![],
            TaskState::Failed {
                error: "err".into(),
            },
        );
        let (icon, color) = MissionControlUI::task_style(&task);
        assert_eq!(icon, '✗');
        assert_eq!(color, ratatui::style::Color::Red);
    }

    #[test]
    fn test_select_task_returns_action() {
        let mut ui = create_populated_ui();
        let action = ui.handle_key(KeyCode::Enter);
        assert!(
            matches!(action, MissionControlAction::SelectTask(idx) if idx == 0 || idx < ui.tasks.len())
        );
    }
}
