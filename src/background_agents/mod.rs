//! Background Agent Dashboard — manage, monitor, and inspect background agents
//!
//! Spawns LLM agents as background tokio tasks, tracks their lifecycle
//! (pending → running → completed/failed), and provides a TUI dashboard
//! view alongside CLI commands for headless management.
//!
//! ## Architecture
//!
//! ```text
//! BackgroundAgentManager
//!   ├── agent_pool: HashMap<Uuid, BackgroundAgent>
//!   ├── event_tx: broadcast channel for status changes
//!   └── log_store: ring buffer of per-agent log lines
//! ```
//!
//! Agents are spawned via [`BackgroundAgentManager::spawn`], which accepts
//! a prompt, optional provider/model override, and an optional file list to
//! constrain the agent's workspace. The manager is thread-safe (all interior
//! state behind `Arc<RwLock<...>>`) and emits status-change events on a
//! broadcast channel that the TUI dashboard subscribes to.

use crate::llm::LlmClient;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

/// Unique agent identifier, human-readable label, and lifecycle stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Pending,
    Running,
    Completed { output: String },
    Failed { error: String },
    Cancelled,
}

/// A single background agent with full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundAgent {
    pub id: Uuid,
    pub name: String,
    pub prompt: String,
    pub provider: String,
    pub model: String,
    pub status: AgentStatus,
    pub progress: u8, // 0-100 approximate percentage
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub log: Vec<String>,  // ring buffer of log lines
    pub tags: Vec<String>, // user-defined tags for grouping/filtering
}

impl BackgroundAgent {
    fn new(name: String, prompt: String, provider: String, model: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            prompt,
            provider,
            model,
            status: AgentStatus::Pending,
            progress: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            log: Vec::with_capacity(256),
            tags: Vec::new(),
        }
    }

    fn log_line(&mut self, line: String) {
        if self.log.len() >= 256 {
            self.log.remove(0);
        }
        self.log
            .push(format!("[{}] {}", Utc::now().format("%H:%M:%S"), line));
    }
}

/// Events emitted by the manager for real-time TUI updates.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    Created(Uuid),
    Started(Uuid),
    Progress(Uuid, u8, String),
    Completed(Uuid, String),
    Failed(Uuid, String),
    Cancelled(Uuid),
    Log(Uuid, String),
}

/// Thread-safe manager for background agents.
pub struct BackgroundAgentManager {
    agents: Arc<RwLock<HashMap<Uuid, BackgroundAgent>>>,
    event_tx: broadcast::Sender<AgentEvent>,
}

impl BackgroundAgentManager {
    /// Create a new empty manager.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Subscribe to real-time agent events (for TUI dashboard).
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// Return a snapshot of all tracked agents.
    pub async fn list(&self) -> Vec<BackgroundAgent> {
        let guard = self.agents.read().await;
        let mut agents: Vec<BackgroundAgent> = guard.values().cloned().collect();
        agents.sort_by_key(|b| std::cmp::Reverse(b.created_at)); // newest first
        agents
    }

    /// Return a single agent by ID.
    pub async fn get(&self, id: &Uuid) -> Option<BackgroundAgent> {
        self.agents.read().await.get(id).cloned()
    }

    /// Spawn a new background agent. Returns the agent UUID.
    pub async fn spawn(
        &self,
        name: String,
        prompt: String,
        provider: Option<String>,
        model: Option<String>,
        tags: Vec<String>,
        llm_client: Arc<LlmClient>,
    ) -> Uuid {
        let resolved_provider = provider
            .clone()
            .unwrap_or_else(|| format!("{:?}", llm_client.config.provider));
        let resolved_model = model.unwrap_or_else(|| llm_client.config.model.clone());

        let mut agent = BackgroundAgent::new(
            name.clone(),
            prompt.clone(),
            resolved_provider,
            resolved_model,
        );
        agent.tags = tags;
        let id = agent.id;
        let event_tx = self.event_tx.clone();
        let agents = self.agents.clone();

        // Insert pending agent
        {
            let mut guard = agents.write().await;
            guard.insert(id, agent);
        }
        let _ = event_tx.send(AgentEvent::Created(id));

        // Spawn the actual LLM work
        tokio::spawn(async move {
            // Mark as running
            {
                let mut guard = agents.write().await;
                if let Some(a) = guard.get_mut(&id) {
                    a.status = AgentStatus::Running;
                    a.started_at = Some(Utc::now());
                    a.log_line("Agent started".into());
                }
            }
            let _ = event_tx.send(AgentEvent::Started(id));

            // Execute the query with timeout
            let timeout = Duration::from_secs(600); // 10 min max
            let result =
                tokio::time::timeout(timeout, llm_client.query_simple(&prompt, None)).await;

            // Update final state
            let mut guard = agents.write().await;
            match result {
                Ok(Ok(output)) => {
                    if let Some(a) = guard.get_mut(&id) {
                        a.status = AgentStatus::Completed {
                            output: output.clone(),
                        };
                        a.progress = 100;
                        a.completed_at = Some(Utc::now());
                        a.log_line("Agent completed successfully".into());
                    }
                    let _ = event_tx.send(AgentEvent::Completed(id, output));
                }
                Ok(Err(e)) => {
                    let err = format!("LLM error: {}", e);
                    if let Some(a) = guard.get_mut(&id) {
                        a.status = AgentStatus::Failed { error: err.clone() };
                        a.completed_at = Some(Utc::now());
                        a.log_line(format!("Agent failed: {}", err));
                    }
                    let _ = event_tx.send(AgentEvent::Failed(id, err));
                }
                Err(_) => {
                    let err = "timeout (600s)".to_string();
                    if let Some(a) = guard.get_mut(&id) {
                        a.status = AgentStatus::Failed { error: err.clone() };
                        a.completed_at = Some(Utc::now());
                        a.log_line(format!("Agent failed: {}", err));
                    }
                    let _ = event_tx.send(AgentEvent::Failed(id, err));
                }
            }
        });

        id
    }

    /// Cancel a running agent by ID. Returns true if the agent existed.
    pub async fn cancel(&self, id: &Uuid) -> bool {
        let mut guard = self.agents.write().await;
        if let Some(agent) = guard.get_mut(id) {
            if matches!(agent.status, AgentStatus::Pending | AgentStatus::Running) {
                agent.status = AgentStatus::Cancelled;
                agent.completed_at = Some(Utc::now());
                agent.log_line("Agent cancelled by user".into());
                let _ = self.event_tx.send(AgentEvent::Cancelled(*id));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Append a log line to a running agent. Returns false if the agent doesn't exist.
    pub async fn log(&self, id: &Uuid, line: String) -> bool {
        let mut guard = self.agents.write().await;
        if let Some(agent) = guard.get_mut(id) {
            agent.log_line(line.clone());
            let _ = self.event_tx.send(AgentEvent::Log(*id, line));
            true
        } else {
            false
        }
    }

    /// Remove an agent from tracking entirely.
    pub async fn remove(&self, id: &Uuid) -> bool {
        let mut guard = self.agents.write().await;
        guard.remove(id).is_some()
    }

    /// Return aggregate statistics.
    pub async fn stats(&self) -> AgentStats {
        let guard = self.agents.read().await;
        let mut stats = AgentStats::default();
        for agent in guard.values() {
            stats.total += 1;
            match &agent.status {
                AgentStatus::Pending => stats.pending += 1,
                AgentStatus::Running => stats.running += 1,
                AgentStatus::Completed { .. } => stats.completed += 1,
                AgentStatus::Failed { .. } => stats.failed += 1,
                AgentStatus::Cancelled => stats.cancelled += 1,
            }
        }
        stats
    }
}

impl Default for BackgroundAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate statistics across all background agents.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStats {
    pub total: u64,
    pub pending: u64,
    pub running: u64,
    pub completed: u64,
    pub failed: u64,
    pub cancelled: u64,
}

impl std::fmt::Display for AgentStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Total: {} | Running: {} | Pending: {} | Completed: {} | Failed: {} | Cancelled: {}",
            self.total, self.running, self.pending, self.completed, self.failed, self.cancelled
        )
    }
}

/// CLI display helper for a single agent.
impl std::fmt::Display for BackgroundAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match &self.status {
            AgentStatus::Pending => "PENDING",
            AgentStatus::Running => "RUNNING",
            AgentStatus::Completed { .. } => "COMPLETED",
            AgentStatus::Failed { .. } => "FAILED",
            AgentStatus::Cancelled => "CANCELLED",
        };
        let elapsed = self
            .started_at
            .map(|s| {
                let end = self.completed_at.unwrap_or_else(Utc::now);
                format!("{}s", (end - s).num_seconds())
            })
            .unwrap_or_else(|| "-".to_string());
        write!(
            f,
            "[{:<9}] {:>4}% | {} | {} ({}) | {}",
            status_str, self.progress, self.name, self.provider, self.model, elapsed,
        )
    }
}

#[cfg(test)]
mod tests;
