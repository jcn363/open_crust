//! Task definitions for the multi-agent orchestrator
//!
//! Defines the core Task type with state machine, dependency graph support,
//! and readiness checking for parallel execution.

use std::collections::HashSet;
use uuid::Uuid;

/// Represents the state of a task in the orchestration pipeline
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    /// Task is waiting to be executed
    Pending,
    /// Task is currently being executed by an agent
    Running { agent_id: String },
    /// Task completed successfully with output
    Completed { output: String },
    /// Task failed with an error message
    Failed { error: String },
}

/// A unit of work to be executed by an agent
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable description
    pub description: String,
    /// IDs of tasks that must complete before this one
    pub dependencies: Vec<Uuid>,
    /// Current state in the pipeline
    pub state: TaskState,
    /// Result output (set when completed or failed)
    pub result: Option<String>,
    /// Type of agent to handle this task
    pub agent_type: String,
}

impl Task {
    /// Create a new pending task
    pub fn new(description: impl Into<String>, agent_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.into(),
            dependencies: Vec::new(),
            state: TaskState::Pending,
            result: None,
            agent_type: agent_type.into(),
        }
    }

    /// Add a dependency on another task (no-op if already present)
    pub fn add_dependency(&mut self, task_id: Uuid) {
        if !self.dependencies.contains(&task_id) {
            self.dependencies.push(task_id);
        }
    }

    /// Check whether all dependencies are satisfied
    pub fn is_ready(&self, completed_ids: &HashSet<Uuid>) -> bool {
        self.dependencies.iter().all(|dep| completed_ids.contains(dep))
    }

    /// Returns true if the task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, TaskState::Completed { .. } | TaskState::Failed { .. })
    }

    /// Returns a summary string for the task
    pub fn summary(&self) -> String {
        let state_str = match &self.state {
            TaskState::Pending => "pending".to_string(),
            TaskState::Running { agent_id } => format!("running (agent: {})", agent_id),
            TaskState::Completed { output } => format!("completed ({} chars)", output.len()),
            TaskState::Failed { error } => format!("failed: {}", error),
        };
        format!("[{}] {} — {}", self.agent_type, self.description, state_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Analyze requirements", "analyst");
        assert_eq!(task.description, "Analyze requirements");
        assert_eq!(task.agent_type, "analyst");
        assert_eq!(task.state, TaskState::Pending);
        assert!(task.result.is_none());
        assert!(task.dependencies.is_empty());
    }

    #[test]
    fn test_add_dependency() {
        let mut task = Task::new("Write code", "coder");
        let dep = Uuid::new_v4();
        task.add_dependency(dep);
        assert_eq!(task.dependencies.len(), 1);
        // duplicate should be a no-op
        task.add_dependency(dep);
        assert_eq!(task.dependencies.len(), 1);
    }

    #[test]
    fn test_is_ready_no_dependencies() {
        let task = Task::new("standalone", "agent");
        assert!(task.is_ready(&HashSet::new()));
    }

    #[test]
    fn test_is_ready_with_dependencies() {
        let mut task = Task::new("dependent", "agent");
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        task.add_dependency(a);
        task.add_dependency(b);

        // not ready yet
        assert!(!task.is_ready(&HashSet::new()));

        // only one met
        let mut partial = HashSet::new();
        partial.insert(a);
        assert!(!task.is_ready(&partial));

        // both met
        let mut all = HashSet::new();
        all.insert(a);
        all.insert(b);
        assert!(task.is_ready(&all));
    }

    #[test]
    fn test_terminal_states() {
        let pending = Task::new("t", "a");
        assert!(!pending.is_terminal());

        let done = Task {
            state: TaskState::Completed { output: "ok".into() },
            ..Task::new("t", "a")
        };
        assert!(done.is_terminal());

        let failed = Task {
            state: TaskState::Failed { error: "err".into() },
            ..Task::new("t", "a")
        };
        assert!(failed.is_terminal());
    }

    #[test]
    fn test_summary_pending() {
        let t = Task::new("inspect", "auditor");
        assert!(t.summary().contains("pending"));
    }

    #[test]
    fn test_summary_completed() {
        let t = Task {
            state: TaskState::Completed { output: "hello".into() },
            ..Task::new("inspect", "auditor")
        };
        assert!(t.summary().contains("completed"));
    }
}
