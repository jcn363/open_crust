//! Mission Control state management

use crate::orchestrator::task::{Task, TaskState};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::dashboard::AgentDashboard;
use super::spaces::SpaceManager;
use super::types::{
    AgentPanel, DashboardStats, MissionControlAction, NodePosition, SpacePanel, ViewMode,
};

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
    /// Agent panel state
    pub agent_panel: AgentPanel,
    /// Space panel state
    pub space_panel: SpacePanel,
    /// Agent dashboard for real-time monitoring
    pub agent_dashboard: AgentDashboard,
    /// Space manager for project organization
    pub space_manager: SpaceManager,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Whether space panel is visible
    pub show_space_panel: bool,
    /// Whether agent dashboard is visible
    pub show_agent_dashboard: bool,
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
            agent_panel: AgentPanel::new(),
            space_panel: SpacePanel::new(),
            agent_dashboard: AgentDashboard::new(),
            space_manager: SpaceManager::new(),
            view_mode: ViewMode::Dag,
            show_space_panel: false,
            show_agent_dashboard: false,
        }
    }

    /// Create a new Mission Control UI with persistence
    pub fn with_persistence(spaces_path: std::path::PathBuf) -> Self {
        let mut ui = Self::new();
        ui.space_manager = SpaceManager::new().with_persistence(spaces_path);
        let _ = ui.space_manager.load();
        ui
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

        let node_width: u16 = super::types::NODE_WIDTH;
        let node_height: u16 = super::types::NODE_HEIGHT;
        let h_gap: u16 = super::types::H_GAP;
        let v_gap: u16 = super::types::V_GAP;

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

    /// Compute dashboard statistics from current tasks and agents
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

        // Get agent dashboard stats
        let agent_stats = self.agent_dashboard.compute_stats();

        self.stats = DashboardStats {
            total,
            pending,
            running,
            completed,
            failed,
            active_agents: agent_stats.active_agents,
            idle_agents: agent_stats.idle_agents,
            failed_agents: agent_stats.failed_agents,
            success_rate: agent_stats.success_rate,
        };
    }

    /// Get the state icon and color for a task
    pub(crate) fn task_style(
        task: &Task,
        theme: &crate::ui::ThemeContext,
    ) -> (char, ratatui::style::Color) {
        match &task.state {
            TaskState::Pending => ('⏳', theme.fg),
            TaskState::Running { .. } => ('▶', theme.warning()),
            TaskState::Completed { .. } => ('✓', theme.success()),
            TaskState::Failed { .. } => ('✗', theme.error()),
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

    /// Create a new space
    pub fn create_space(&mut self, name: String) -> String {
        let space = self.space_manager.create_space(name);
        let id = space.id.clone();
        self.space_panel.spaces = self
            .space_manager
            .list_spaces(true)
            .into_iter()
            .cloned()
            .collect();
        id
    }

    /// Set the active space
    pub fn set_active_space(&mut self, space_id: Option<String>) {
        self.space_manager.set_active_space(space_id.clone());
        self.space_panel.spaces = self
            .space_manager
            .list_spaces(true)
            .into_iter()
            .cloned()
            .collect();
    }

    /// Get the active space
    pub fn get_active_space(&self) -> Option<&super::spaces::Space> {
        self.space_manager.get_active_space()
    }

    /// List all spaces
    pub fn list_spaces(&self, include_archived: bool) -> Vec<&super::spaces::Space> {
        self.space_manager.list_spaces(include_archived)
    }

    /// Delete a space
    pub fn delete_space(&mut self, space_id: &str) -> bool {
        let result = self.space_manager.delete_space(space_id);
        self.space_panel.spaces = self
            .space_manager
            .list_spaces(true)
            .into_iter()
            .cloned()
            .collect();
        result
    }

    /// Add an agent to a space
    pub fn add_agent_to_space(&mut self, space_id: &str, agent_id: &str) {
        if let Some(space) = self.space_manager.get_space_mut(space_id) {
            space.add_agent(agent_id.to_string());
        }
    }

    /// Remove an agent from a space
    pub fn remove_agent_from_space(&mut self, space_id: &str, agent_id: &str) {
        if let Some(space) = self.space_manager.get_space_mut(space_id) {
            space.remove_agent(agent_id);
        }
    }

    /// Get agent dashboard reference
    pub fn agent_dashboard(&self) -> &super::dashboard::AgentDashboard {
        &self.agent_dashboard
    }

    /// Get mutable agent dashboard reference
    pub fn agent_dashboard_mut(&mut self) -> &mut super::dashboard::AgentDashboard {
        &mut self.agent_dashboard
    }

    /// Toggle space panel visibility
    pub fn toggle_space_panel(&mut self) {
        self.show_space_panel = !self.show_space_panel;
    }

    /// Toggle agent dashboard visibility
    pub fn toggle_agent_dashboard(&mut self) {
        self.show_agent_dashboard = !self.show_agent_dashboard;
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: super::types::ViewMode) {
        self.view_mode = mode;
    }

    /// Get current view mode
    pub fn view_mode(&self) -> &super::types::ViewMode {
        &self.view_mode
    }

    /// Save spaces to disk
    pub fn save_spaces(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.space_manager.save()
    }

    /// Load spaces from disk
    pub fn load_spaces(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.space_manager.load()?;
        self.space_panel.spaces = self
            .space_manager
            .list_spaces(true)
            .into_iter()
            .cloned()
            .collect();
        Ok(())
    }

    /// Search spaces
    pub fn search_spaces(&self, query: &str) -> Vec<&super::spaces::Space> {
        self.space_manager.search(query)
    }

    /// Get recently active spaces
    pub fn recently_active_spaces(&self, limit: usize) -> Vec<&super::spaces::Space> {
        self.space_manager.recently_active(limit)
    }
}

impl Default for MissionControlUI {
    fn default() -> Self {
        Self::new()
    }
}
