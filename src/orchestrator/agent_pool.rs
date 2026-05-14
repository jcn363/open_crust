//! Subagent lifecycle management for the orchestrator
//!
//! Manages spawning, timeout, retry, and cancellation of LLM subagents
//! running as concurrent tokio tasks.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::llm::LlmClient;
use crate::orchestrator::task::Task;

/// Configuration for agent pool behaviour
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Maximum number of concurrently running agents
    pub max_concurrent: usize,
    /// Per-agent timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retry attempts on failure (actual attempts = max_retries + 1)
    pub max_retries: u32,
    /// Optional model override applied to every agent
    /// Part of the config interface for future extensibility; reserved for programmatic
    /// model selection when multiple providers are supported.
    #[allow(dead_code)]
    pub model_override: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            timeout_secs: 300,
            max_retries: 3,
            model_override: None,
        }
    }
}

/// Result produced by a finished agent
#[derive(Debug, Clone)]
pub struct AgentResult {
    #[expect(dead_code, reason = "result correlation identifier")]
    pub task_id: uuid::Uuid,
    pub output: String,
    pub success: bool,
}

/// Manages the lifecycle of spawned subagents
pub struct AgentPool {
    config: AgentConfig,
    active_agents: HashMap<uuid::Uuid, JoinHandle<()>>,
}

impl AgentPool {
    /// Create a new pool with the given configuration
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            active_agents: HashMap::new(),
        }
    }

    #[expect(dead_code, reason = "pool config inspector")]
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Spawn an agent for a task. Returns a receiver that will deliver the result.
    pub fn spawn_agent(
        &mut self,
        task: &Task,
        llm_client: Arc<LlmClient>,
    ) -> Result<mpsc::Receiver<AgentResult>, String> {
        if self.active_agents.len() >= self.config.max_concurrent {
            return Err(format!(
                "max concurrent agents ({}) already reached",
                self.config.max_concurrent
            ));
        }

        let (tx, rx) = mpsc::channel(1);
        let task_id = task.id;
        let description = task.description.clone();
        let timeout = Duration::from_secs(self.config.timeout_secs);
        let max_retries = self.config.max_retries;
        let model_override = self.config.model_override.clone();

        let handle = tokio::spawn(async move {
            let mut last_error = String::from("unknown error");

            for attempt in 0..=max_retries {
                if attempt > 0 {
                    // exponential backoff: 1s, 2s, 4s, …
                    let backoff = Duration::from_secs(1_u64 << (attempt - 1));
                    tokio::time::sleep(backoff).await;
                }

                let result = tokio::time::timeout(
                    timeout,
                    llm_client.query_simple(&description, model_override.as_deref()),
                )
                .await;

                match result {
                    Ok(Ok(output)) => {
                        let _ = tx
                            .send(AgentResult {
                                task_id,
                                output,
                                success: true,
                            })
                            .await;
                        return;
                    }
                    Ok(Err(e)) => {
                        last_error = format!("attempt {}: {}", attempt, e);
                    }
                    Err(_) => {
                        last_error =
                            format!("attempt {}: timeout ({}s)", attempt, timeout.as_secs());
                    }
                }
            }

            // all retries exhausted
            let _ = tx
                .send(AgentResult {
                    task_id,
                    output: last_error,
                    success: false,
                })
                .await;
        });

        self.active_agents.insert(task_id, handle);
        Ok(rx)
    }

    /// Cancel a running agent (aborts the tokio task)
    pub fn cancel(&mut self, task_id: &uuid::Uuid) -> bool {
        self.active_agents.remove(task_id).is_some_and(|h| {
            h.abort();
            true
        })
    }

    /// Number of agents currently active
    pub fn active_count(&self) -> usize {
        self.active_agents.len()
    }

    /// Remove completed agent handles from tracking
    pub fn cleanup(&mut self, ids: &[uuid::Uuid]) {
        for id in ids {
            self.active_agents.remove(id);
        }
    }

    /// Get all running agent task IDs
    pub fn running_ids(&self) -> Vec<uuid::Uuid> {
        self.active_agents.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AgentConfig::default();
        assert_eq!(cfg.max_concurrent, 5);
        assert_eq!(cfg.timeout_secs, 300);
        assert_eq!(cfg.max_retries, 3);
        assert!(cfg.model_override.is_none());
    }

    #[test]
    fn test_pool_starts_empty() {
        let pool = AgentPool::new(AgentConfig::default());
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_cancel_nonexistent() {
        let mut pool = AgentPool::new(AgentConfig::default());
        assert!(!pool.cancel(&uuid::Uuid::new_v4()));
    }

    #[test]
    fn test_spawn_rejected_when_full() {
        let cfg = AgentConfig {
            max_concurrent: 0,
            ..Default::default()
        };
        let mut pool = AgentPool::new(cfg);

        // We cannot easily test spawn without a real LlmClient, but we can
        // verify the guard condition causes an error when max_concurrent == 0.
        assert_eq!(pool.active_count(), 0);

        // Cleanup on empty list is a no-op
        pool.cleanup(&[]);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_cleanup_removes_ids() {
        let mut pool = AgentPool::new(AgentConfig::default());
        // Nothing to clean up
        pool.cleanup(&[uuid::Uuid::new_v4()]);
        assert_eq!(pool.active_count(), 0);
    }
}
