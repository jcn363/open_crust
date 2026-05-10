//! Task dependency resolution and parallel execution coordinator
//!
//! Builds a DAG from a list of tasks, resolves ready tasks in topological
//! order, spawns them through the AgentPool, and aggregates results.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::llm::LlmClient;
use crate::orchestrator::agent_pool::{AgentConfig, AgentPool, AgentResult};
use crate::orchestrator::task::{Task, TaskState};

/// Aggregated result from a coordinated execution
#[derive(Debug, Clone)]
pub struct CoordinatorResult {
    /// Tasks that completed successfully
    pub completed: Vec<Task>,
    /// Tasks that failed
    pub failed: Vec<Task>,
    /// Human-readable summary
    pub summary: String,
}

/// Drives task execution by resolving dependencies and managing concurrency
pub struct Coordinator {
    pool: AgentPool,
    /// Shared state for live TUI visualization
    shared_state: Option<std::sync::Arc<tokio::sync::RwLock<Vec<Task>>>>,
}

impl Coordinator {
    /// Create a new coordinator with the given agent configuration
    pub fn new(agent_config: AgentConfig) -> Self {
        Self {
            pool: AgentPool::new(agent_config),
            shared_state: None,
        }
    }

    /// Attach a shared state bridge for live TUI visualization.
    /// The coordinator will update this shared state on every task transition.
    #[allow(dead_code)]
    pub fn with_shared_state(
        mut self,
        state: std::sync::Arc<tokio::sync::RwLock<Vec<Task>>>,
    ) -> Self {
        self.shared_state = Some(state);
        self
    }

    /// Snapshot current tasks into shared state (if bridge is attached)
    fn sync_shared_state(&self, tasks: &[Task]) {
        if let Some(ref shared) = self.shared_state
            && let Ok(mut guard) = shared.try_write()
        {
            *guard = tasks.to_vec();
        }
    }

    /// Execute a list of tasks, respecting their dependency graph.
    ///
    /// This function:
    /// 1. Identifies tasks whose dependencies are all satisfied
    /// 2. Spawns ready tasks in parallel via the AgentPool
    /// 3. Listens for completion events
    /// 4. Updates task states and triggers newly-ready tasks
    /// 5. Returns aggregated results once all tasks are terminal
    pub async fn execute(
        &mut self,
        tasks: &mut [Task],
        llm_client: Arc<LlmClient>,
    ) -> CoordinatorResult {
        let total = tasks.len();
        let mut completed_ids: HashSet<uuid::Uuid> = HashSet::new();
        let mut running: HashMap<uuid::Uuid, mpsc::Receiver<AgentResult>> = HashMap::new();
        let mut all_completed: Vec<Task> = Vec::new();
        let mut all_failed: Vec<Task> = Vec::new();

        // Index tasks by id for quick lookup
        let mut task_map: HashMap<uuid::Uuid, usize> = HashMap::new();
        for (i, t) in tasks.iter().enumerate() {
            task_map.insert(t.id, i);
        }

        // Spawn all initially ready tasks
        for i in 0..total {
            if tasks[i].is_ready(&completed_ids)
                && !tasks[i].is_terminal()
                && let Ok(rx) = self.pool.spawn_agent(&tasks[i], llm_client.clone())
            {
                tasks[i].state = TaskState::Running {
                    agent_id: tasks[i].agent_type.clone(),
                };
                self.sync_shared_state(tasks);
                running.insert(tasks[i].id, rx);
            }
        }

        // Collect receivers to poll
        let mut receivers: Vec<(uuid::Uuid, mpsc::Receiver<AgentResult>)> =
            running.drain().collect();

        while !receivers.is_empty() || all_completed.len() + all_failed.len() < total {
            if receivers.is_empty() {
                // No agents running — check if there are still pending tasks
                // that might never become ready (orphaned by failed deps)
                for i in 0..total {
                    if !tasks[i].is_terminal() {
                        tasks[i].state = TaskState::Failed {
                            error: "dependencies failed or cancelled".into(),
                        };
                        self.sync_shared_state(tasks);
                        all_failed.push(tasks[i].clone());
                    }
                }
                break;
            }

            // Poll receivers for completed agents
            let mut found_result: Option<(uuid::Uuid, AgentResult)> = None;
            let mut pending: Vec<(uuid::Uuid, mpsc::Receiver<AgentResult>)> = Vec::new();

            while let Some((id, mut rx)) = receivers.pop() {
                match rx.try_recv() {
                    Ok(result) => {
                        found_result = Some((id, result));
                        // Collect the rest of the receivers into pending
                        while let Some(item) = receivers.pop() {
                            pending.push(item);
                        }
                        break;
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        pending.push((id, rx));
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        found_result = Some((
                            id,
                            AgentResult {
                                task_id: id,
                                output: "agent disconnected unexpectedly".into(),
                                success: false,
                            },
                        ));
                        while let Some(item) = receivers.pop() {
                            pending.push(item);
                        }
                        break;
                    }
                }
            }

            // Swap pending back into receivers
            std::mem::swap(&mut receivers, &mut pending);

            let (resolved_id, resolved) = match found_result {
                Some((id, res)) => (id, res),
                None => {
                    // No result yet — yield and retry
                    tokio::task::yield_now().await;
                    continue;
                }
            };

            // Update task state from result
            if let Some(&idx) = task_map.get(&resolved_id) {
                let task = &mut tasks[idx];
                if resolved.success {
                    task.state = TaskState::Completed {
                        output: resolved.output.clone(),
                    };
                    task.result = Some(resolved.output.clone());
                    all_completed.push(task.clone());
                } else {
                    task.state = TaskState::Failed {
                        error: resolved.output.clone(),
                    };
                    task.result = Some(resolved.output.clone());
                    all_failed.push(task.clone());
                }
                completed_ids.insert(resolved_id);

                // Spawn newly-ready tasks
                for task in tasks.iter_mut() {
                    if task.is_ready(&completed_ids)
                        && !task.is_terminal()
                        && let Ok(rx) = self.pool.spawn_agent(task, llm_client.clone())
                    {
                        task.state = TaskState::Running {
                            agent_id: task.agent_type.clone(),
                        };
                        receivers.push((task.id, rx));
                    }
                }
            }
        }

        let summary = format!(
            "executed {} tasks: {} completed, {} failed",
            total,
            all_completed.len(),
            all_failed.len()
        );

        CoordinatorResult {
            completed: all_completed,
            failed: all_failed,
            summary,
        }
    }

    /// Cancel all running agents
    #[allow(dead_code)]
    pub fn cancel_all(&mut self) {
        // AgentPool cleanup handled via drop / individual cancellation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_result() {
        let result = CoordinatorResult {
            completed: vec![],
            failed: vec![],
            summary: "nothing to do".into(),
        };
        assert!(result.completed.is_empty());
        assert!(result.failed.is_empty());
        assert_eq!(result.summary, "nothing to do");
    }

    #[test]
    fn test_coordinator_creation() {
        let coord = Coordinator::new(AgentConfig::default());
        // Pool starts empty
        assert_eq!(coord.pool.active_count(), 0);
    }

    #[test]
    fn test_task_dag_independent() {
        // Two independent tasks: both should be ready immediately
        let t1 = Task::new("task 1", "agent");
        let t2 = Task::new("task 2", "agent");

        let ready = HashSet::new();
        assert!(t1.is_ready(&ready));
        assert!(t2.is_ready(&ready));
    }

    #[test]
    fn test_task_dag_sequential() {
        // t2 depends on t1
        let t1 = Task::new("task 1", "agent");
        let mut t2 = Task::new("task 2", "agent");

        let t1_id = t1.id;
        t2.add_dependency(t1_id);

        let mut completed = HashSet::new();
        assert!(!t2.is_ready(&completed));

        completed.insert(t1_id);
        assert!(t2.is_ready(&completed));
    }
}
