//! Mission Control module for visualizing the orchestrator task DAG
//! Provides TUI (ratatui) for real-time task graph monitoring
//! Enhanced with Spaces/Projects, Agent Dashboard, Artifacts System, Workflows, and Scheduler

mod artifacts;
mod dashboard;
mod render;
mod scheduler;
mod spaces;
mod state;
mod types;
mod workflows;

pub use artifacts::{Artifact, ArtifactManager, ArtifactType};
pub use dashboard::{AgentDashboard, AgentState, AgentStatus, DashboardStats};
pub use scheduler::{ScheduleType, ScheduledTask, TaskScheduler};
pub use spaces::{Space, SpaceManager};
pub use state::MissionControlUI;
pub use types::MissionControlAction;
pub use workflows::{Workflow, WorkflowCategory, WorkflowManager, WorkflowParameter};

#[cfg(test)]
mod tests;
