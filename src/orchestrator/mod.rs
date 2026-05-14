//! Multi-Agent Orchestrator
//!
//! Top-level module providing the `Orchestrator` — the main entry point
//! for spawning and coordinating multiple LLM subagents.
//!
//! ## Architecture
//!
//! ```text
//! Orchestrator
//!   ├── coordinator  →  resolves DAG, drives execution
//!   └── agent_pool   →  manages concurrent agent lifecycle
//! ```
//!
//! Tasks are created from a user request, organised into a dependency
//! graph (DAG), and executed in topological order with maximal parallelism.

pub mod agent_pool;
pub mod coordinator;
pub mod task;

use std::sync::Arc;

pub use agent_pool::AgentConfig;
pub use coordinator::CoordinatorResult;
pub use task::Task;

use crate::config::Config;
use crate::llm::LlmClient;
use crate::orchestrator::coordinator::Coordinator;

/// High-level orchestrator for multi-agent task execution
pub struct Orchestrator {
    coordinator: Coordinator,
    #[allow(dead_code, reason = "accessible to subagent coordination")]
    pub(crate) config: Arc<Config>,
}

impl Orchestrator {
    /// Create a new orchestrator from the application config
    pub fn new(config: Arc<Config>) -> Self {
        let agent_config = AgentConfig {
            max_concurrent: config.subagent_max_concurrent.unwrap_or(5),
            timeout_secs: config.subagent_timeout_secs.unwrap_or(300),
            max_retries: 3,
            model_override: config
                .subagent_config
                .as_ref()
                .and_then(|sc| sc.default_model.clone()),
        };

        Self {
            coordinator: Coordinator::new(agent_config),
            config,
        }
    }

    /// Execute a natural-language request by decomposing it into subtasks
    /// and running them through the coordinator.
    ///
    /// For now, a simple rule-based decomposition is used:
    /// - If the request mentions keywords like "research", "search", a
    ///   research sub-task is created first.
    /// - If "code", "implement", "write" appears, a coding task is added.
    /// - If "test" appears, a testing task is added (depends on code).
    /// - Everything else is run as a single agent call.
    pub async fn execute_request(
        &mut self,
        request: &str,
        llm_client: Arc<LlmClient>,
    ) -> CoordinatorResult {
        let mut tasks = self.decompose_request(request);
        self.coordinator.execute(&mut tasks, llm_client).await
    }

    /// Simple rule-based task decomposition.
    /// Returns a list of tasks with appropriate dependencies.
    fn decompose_request(&self, request: &str) -> Vec<Task> {
        let lower = request.to_lowercase();
        let mut tasks: Vec<Task> = Vec::new();

        // Research phase
        if lower.contains("research") || lower.contains("search") || lower.contains("find") {
            tasks.push(Task::new(format!("Research: {}", request), "researcher"));
        }

        // Code/implementation phase
        if lower.contains("code")
            || lower.contains("implement")
            || lower.contains("write")
            || lower.contains("build")
        {
            let research_id = tasks.first().map(|t| t.id);
            let mut code_task = Task::new(format!("Implement: {}", request), "coder");
            if let Some(id) = research_id {
                code_task.add_dependency(id);
            }
            tasks.push(code_task);
        }

        // Test phase (depends on code)
        if lower.contains("test") || lower.contains("verify") || lower.contains("validate") {
            // Find the code task
            let code_id = tasks.iter().find(|t| t.agent_type == "coder").map(|t| t.id);
            let research_id = tasks.first().map(|t| t.id);
            let mut test_task = Task::new(format!("Test: {}", request), "tester");
            if let Some(id) = code_id {
                test_task.add_dependency(id);
            } else if let Some(id) = research_id {
                test_task.add_dependency(id);
            }
            tasks.push(test_task);
        }

        // If nothing matched, return a single generic task
        if tasks.is_empty() {
            tasks.push(Task::new(request.to_string(), "general"));
        }

        tasks
    }

    /// Cancel all running agents
    ///
    /// High-level public API for bulk agent cancellation. Delegates to the
    /// Coordinator which manages agent pool lifecycle. Part of the Orchestrator
    /// public interface for feature-complete agent control.
    #[allow(dead_code, reason = "public API for orchestrator cancellation")]
    pub fn cancel_all(&mut self) {
        self.coordinator.cancel_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_decompose_code_request() {
        let config = Arc::new(Config::default());
        let orch = Orchestrator::new(config);
        let tasks = orch.decompose_request("Write a Rust HTTP server");
        assert!(!tasks.is_empty());
        assert!(tasks.iter().any(|t| t.agent_type == "coder"));
    }

    #[test]
    fn test_decompose_research_request() {
        let config = Arc::new(Config::default());
        let orch = Orchestrator::new(config);
        let tasks = orch.decompose_request("Research quantum computing");
        assert!(!tasks.is_empty());
        assert!(tasks.iter().any(|t| t.agent_type == "researcher"));
    }

    #[test]
    fn test_decompose_full_workflow() {
        let config = Arc::new(Config::default());
        let orch = Orchestrator::new(config);
        let tasks =
            orch.decompose_request("Research and implement a sorting algorithm, then test it");
        assert!(tasks.len() >= 3);
        let types: Vec<&str> = tasks.iter().map(|t| t.agent_type.as_str()).collect();
        assert!(types.contains(&"researcher"));
        assert!(types.contains(&"coder"));
        assert!(types.contains(&"tester"));
    }

    #[test]
    fn test_decompose_generic_fallback() {
        let config = Arc::new(Config::default());
        let orch = Orchestrator::new(config);
        let tasks = orch.decompose_request("Hello world");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].agent_type, "general");
    }

    #[test]
    fn test_orchestrator_dependencies_ordering() {
        let config = Arc::new(Config::default());
        let orch = Orchestrator::new(config);
        let tasks = orch.decompose_request("Research and implement feature X");

        // Research should come first
        assert!(tasks[0].agent_type == "researcher");
        // Code should depend on research
        if tasks.len() > 1 {
            let code = &tasks[1];
            assert_eq!(code.agent_type, "coder");
            assert!(code.dependencies.contains(&tasks[0].id));
        }
    }
}
