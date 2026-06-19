//! Agent Dashboard for real-time agent monitoring
//! Inspired by Devin Desktop's Agent Command Center

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Agent status information
#[derive(Debug, Clone)]
pub struct AgentStatus {
    /// Agent unique identifier
    pub id: String,
    /// Agent name/description
    pub name: String,
    /// Current status
    pub state: AgentState,
    /// When the agent was started
    pub started_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Current task being worked on
    pub current_task: Option<String>,
    /// Progress percentage (0-100)
    pub progress: Option<u8>,
    /// Resource usage
    pub resources: AgentResources,
    /// Recent log entries
    pub logs: Vec<LogEntry>,
    /// Associated space ID
    pub space_id: Option<String>,
}

/// Agent states
#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    /// Agent is starting up
    Starting,
    /// Agent is idle, waiting for tasks
    Idle,
    /// Agent is actively working on a task
    Working,
    /// Agent is paused
    Paused,
    /// Agent has completed its task
    Completed,
    /// Agent encountered an error
    Failed(String),
    /// Agent was cancelled
    Cancelled,
}

impl AgentState {
    /// Get status icon for display
    pub fn icon(&self) -> &str {
        match self {
            AgentState::Starting => "🔄",
            AgentState::Idle => "💤",
            AgentState::Working => "⚡",
            AgentState::Paused => "⏸️",
            AgentState::Completed => "✅",
            AgentState::Failed(_) => "❌",
            AgentState::Cancelled => "🚫",
        }
    }

    /// Get status color name
    pub fn color(&self) -> &str {
        match self {
            AgentState::Starting => "blue",
            AgentState::Idle => "gray",
            AgentState::Working => "green",
            AgentState::Paused => "yellow",
            AgentState::Completed => "green",
            AgentState::Failed(_) => "red",
            AgentState::Cancelled => "red",
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Default)]
pub struct AgentResources {
    /// CPU usage percentage (0-100)
    pub cpu_percent: Option<f32>,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// Number of API calls made
    pub api_calls: u32,
    /// Tokens consumed
    pub tokens_used: u64,
    /// Storage used in bytes
    pub storage_bytes: u64,
}

/// Log entry for agent activity
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Optional source/module
    pub source: Option<String>,
}

/// Log levels
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Get icon for display
    pub fn icon(&self) -> &str {
        match self {
            LogLevel::Debug => "🔍",
            LogLevel::Info => "ℹ️",
            LogLevel::Warn => "⚠️",
            LogLevel::Error => "❌",
        }
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Default)]
pub struct DashboardStats {
    /// Total number of agents
    pub total_agents: usize,
    /// Number of active (working) agents
    pub active_agents: usize,
    /// Number of idle agents
    pub idle_agents: usize,
    /// Number of failed agents
    pub failed_agents: usize,
    /// Total tasks completed
    pub total_tasks_completed: u64,
    /// Total tasks failed
    pub total_tasks_failed: u64,
    /// Total API calls across all agents
    pub total_api_calls: u64,
    /// Total tokens consumed
    pub total_tokens_used: u64,
    /// Average task duration
    pub avg_task_duration: Duration,
    /// Success rate (0-100)
    pub success_rate: f32,
}

/// Agent Dashboard for monitoring and managing agents
#[derive(Debug, Default)]
pub struct AgentDashboard {
    /// All agent statuses indexed by agent ID
    agents: HashMap<String, AgentStatus>,
    /// Currently selected agent index
    selected_index: usize,
    /// Scroll offset for agent list
    #[allow(dead_code, reason = "Used in UI rendering")]
    scroll_offset: usize,
    /// Whether to show logs panel
    show_logs: bool,
    /// Filter by state
    state_filter: Option<AgentState>,
    /// Filter by space
    space_filter: Option<String>,
}

impl AgentDashboard {
    /// Create a new dashboard
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update an agent
    pub fn update_agent(&mut self, status: AgentStatus) {
        self.agents.insert(status.id.clone(), status);
    }

    /// Remove an agent
    pub fn remove_agent(&mut self, agent_id: &str) -> Option<AgentStatus> {
        self.agents.remove(agent_id)
    }

    /// Get an agent by ID
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentStatus> {
        self.agents.get(agent_id)
    }

    /// Get a mutable reference to an agent by ID
    pub fn get_agent_mut(&mut self, agent_id: &str) -> Option<&mut AgentStatus> {
        self.agents.get_mut(agent_id)
    }

    /// Get all agents (optionally filtered)
    pub fn list_agents(&self, include_filtered: bool) -> Vec<&AgentStatus> {
        self.agents
            .values()
            .filter(|a| {
                if include_filtered {
                    true
                } else if let Some(ref state_filter) = self.state_filter {
                    a.state == *state_filter
                } else if let Some(ref space_filter) = self.space_filter {
                    a.space_id.as_ref() == Some(space_filter)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Set selected agent index
    pub fn set_selected(&mut self, index: usize) {
        self.selected_index = index;
    }

    /// Get selected agent
    pub fn get_selected(&self) -> Option<&AgentStatus> {
        let agents = self.list_agents(false);
        agents.get(self.selected_index).copied()
    }

    /// Toggle logs panel visibility
    pub fn toggle_logs(&mut self) {
        self.show_logs = !self.show_logs;
    }

    /// Set state filter
    pub fn set_state_filter(&mut self, filter: Option<AgentState>) {
        self.state_filter = filter;
        self.selected_index = 0;
    }

    /// Set space filter
    pub fn set_space_filter(&mut self, filter: Option<String>) {
        self.space_filter = filter;
        self.selected_index = 0;
    }

    /// Compute dashboard statistics
    pub fn compute_stats(&self) -> DashboardStats {
        let agents: Vec<&AgentStatus> = self.agents.values().collect();
        let total = agents.len();

        let active = agents
            .iter()
            .filter(|a| a.state == AgentState::Working)
            .count();
        let idle = agents
            .iter()
            .filter(|a| a.state == AgentState::Idle)
            .count();
        let failed = agents
            .iter()
            .filter(|a| matches!(a.state, AgentState::Failed(_)))
            .count();

        let total_api_calls: u64 = agents.iter().map(|a| a.resources.api_calls as u64).sum();
        let total_tokens: u64 = agents.iter().map(|a| a.resources.tokens_used).sum();

        // Calculate success rate from completed vs failed
        let completed = agents
            .iter()
            .filter(|a| a.state == AgentState::Completed)
            .count();
        let success_rate = if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        // Calculate average task duration from completed agents
        let avg_task_duration = {
            let completed_durations: Vec<Duration> = agents
                .iter()
                .filter(|a| a.state == AgentState::Completed)
                .filter_map(|a| a.last_activity.duration_since(a.started_at).ok())
                .collect();

            if completed_durations.is_empty() {
                Duration::ZERO
            } else {
                let total_duration: Duration = completed_durations.iter().sum();
                total_duration / completed_durations.len() as u32
            }
        };

        DashboardStats {
            total_agents: total,
            active_agents: active,
            idle_agents: idle,
            failed_agents: failed,
            total_tasks_completed: completed as u64,
            total_tasks_failed: failed as u64,
            total_api_calls,
            total_tokens_used: total_tokens,
            avg_task_duration,
            success_rate,
        }
    }

    /// Add log entry to an agent
    pub fn add_log(&mut self, agent_id: &str, entry: LogEntry) {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.logs.push(entry);
            // Keep only last 100 log entries
            if agent.logs.len() > 100 {
                agent.logs.drain(0..agent.logs.len() - 100);
            }
            agent.last_activity = SystemTime::now();
        }
    }

    /// Get agent count
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Get agents by state
    pub fn agents_by_state(&self, state: &AgentState) -> Vec<&AgentStatus> {
        self.agents.values().filter(|a| a.state == *state).collect()
    }

    /// Get agents by space
    pub fn agents_by_space(&self, space_id: &str) -> Vec<&AgentStatus> {
        self.agents
            .values()
            .filter(|a| a.space_id.as_deref() == Some(space_id))
            .collect()
    }

    /// Clear all agents
    pub fn clear(&mut self) {
        self.agents.clear();
        self.selected_index = 0;
    }
}

impl AgentStatus {
    /// Create a new agent status
    pub fn new(id: String, name: String) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            name,
            state: AgentState::Starting,
            started_at: now,
            last_activity: now,
            current_task: None,
            progress: None,
            resources: AgentResources::default(),
            logs: Vec::new(),
            space_id: None,
        }
    }

    /// Update agent state
    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
        self.last_activity = SystemTime::now();
    }

    /// Set current task
    pub fn set_task(&mut self, task: Option<String>) {
        self.current_task = task;
        self.last_activity = SystemTime::now();
    }

    /// Update progress
    pub fn set_progress(&mut self, progress: Option<u8>) {
        self.progress = progress.map(|p| p.min(100));
        self.last_activity = SystemTime::now();
    }

    /// Record API call
    pub fn record_api_call(&mut self, tokens: u64) {
        self.resources.api_calls += 1;
        self.resources.tokens_used += tokens;
        self.last_activity = SystemTime::now();
    }

    /// Check if agent is active (Working or Starting)
    pub fn is_active(&self) -> bool {
        matches!(self.state, AgentState::Working | AgentState::Starting)
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.started_at)
            .unwrap_or_default()
    }

    /// Get time since last activity
    pub fn idle_time(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.last_activity)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_creation() {
        let status = AgentStatus::new("agent1".to_string(), "Test Agent".to_string());
        assert_eq!(status.id, "agent1");
        assert_eq!(status.name, "Test Agent");
        assert_eq!(status.state, AgentState::Starting);
    }

    #[test]
    fn test_agent_state_icons() {
        assert_eq!(AgentState::Starting.icon(), "🔄");
        assert_eq!(AgentState::Working.icon(), "⚡");
        assert_eq!(AgentState::Completed.icon(), "✅");
        assert_eq!(AgentState::Failed("error".to_string()).icon(), "❌");
    }

    #[test]
    fn test_dashboard_stats() {
        let mut dashboard = AgentDashboard::new();

        let mut agent1 = AgentStatus::new("1".to_string(), "Agent 1".to_string());
        agent1.set_state(AgentState::Working);
        dashboard.update_agent(agent1);

        let mut agent2 = AgentStatus::new("2".to_string(), "Agent 2".to_string());
        agent2.set_state(AgentState::Completed);
        dashboard.update_agent(agent2);

        let stats = dashboard.compute_stats();
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.active_agents, 1);
        assert_eq!(stats.total_tasks_completed, 1);
    }
}
