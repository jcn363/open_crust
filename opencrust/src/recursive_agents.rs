//! Recursive sub-agents — agents can spawn their own sub-agents up to 5 levels deep.
//!
//! Provides a tree-structured agent hierarchy with depth enforcement and
//! lifecycle management. Inspired by Claude Code's multi-level sub-agent system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum recursion depth for sub-agents.
pub const MAX_AGENT_DEPTH: u8 = 5;

/// Status of a recursive agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecursiveAgentStatus {
    Spawning,
    Running,
    Completed,
    Failed,
    Cancelled,
    MaxDepthReached,
}

/// A single agent node in the recursive tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub depth: u8,
    pub prompt: String,
    pub status: RecursiveAgentStatus,
    pub result: Option<String>,
    pub children: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl AgentNode {
    fn new(id: String, parent_id: Option<String>, depth: u8, prompt: String) -> Self {
        Self {
            id,
            parent_id,
            depth,
            prompt,
            status: RecursiveAgentStatus::Spawning,
            result: None,
            children: Vec::new(),
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}

/// Manager for recursive sub-agents with depth enforcement.
pub struct RecursiveAgentManager {
    agents: HashMap<String, AgentNode>,
    max_depth: u8,
}

impl RecursiveAgentManager {
    /// Create a new manager with default max depth of 5.
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            max_depth: MAX_AGENT_DEPTH,
        }
    }

    /// Spawn a new agent, optionally as a child of an existing agent.
    /// Returns the agent ID, or an error if max depth is reached.
    pub fn spawn_agent(&mut self, parent_id: Option<&str>, prompt: &str) -> Result<String, String> {
        let depth = if let Some(pid) = parent_id {
            let parent = self.agents.get(pid).ok_or("Parent agent not found")?;
            if parent.depth + 1 >= self.max_depth {
                return Err(format!(
                    "Max depth {} reached. Cannot spawn sub-agent at depth {}.",
                    self.max_depth,
                    parent.depth + 1
                ));
            }
            parent.depth + 1
        } else {
            0
        };

        let id = format!("agent-{}", uuid_simple());
        let mut node = AgentNode::new(
            id.clone(),
            parent_id.map(|s| s.to_string()),
            depth,
            prompt.to_string(),
        );
        node.status = RecursiveAgentStatus::Running;

        // Update parent's children list
        if let Some(pid) = parent_id {
            if let Some(parent) = self.agents.get_mut(pid) {
                parent.children.push(id.clone());
            }
        }

        self.agents.insert(id.clone(), node);
        Ok(id)
    }

    /// Get a reference to an agent by ID.
    pub fn get_agent(&self, id: &str) -> Option<&AgentNode> {
        self.agents.get(id)
    }

    /// Get all direct children of an agent.
    pub fn get_children(&self, id: &str) -> Vec<&AgentNode> {
        self.agents
            .values()
            .filter(|a| a.parent_id.as_deref() == Some(id))
            .collect()
    }

    /// Get top-level agents (depth 0, no parent).
    pub fn get_tree_roots(&self) -> Vec<&AgentNode> {
        self.agents
            .values()
            .filter(|a| a.parent_id.is_none())
            .collect()
    }

    /// Update an agent's status.
    pub fn update_status(&mut self, id: &str, status: RecursiveAgentStatus) -> Result<(), String> {
        let agent = self
            .agents
            .get_mut(id)
            .ok_or_else(|| format!("Agent '{}' not found", id))?;
        agent.status = status;
        if agent.status == RecursiveAgentStatus::Completed
            || agent.status == RecursiveAgentStatus::Failed
            || agent.status == RecursiveAgentStatus::Cancelled
        {
            agent.completed_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Set result text for a completed agent.
    pub fn set_result(&mut self, id: &str, result: String) -> Result<(), String> {
        let agent = self
            .agents
            .get_mut(id)
            .ok_or_else(|| format!("Agent '{}' not found", id))?;
        agent.result = Some(result);
        Ok(())
    }

    /// Count agents at a specific depth.
    pub fn get_depth_count(&self, depth: u8) -> usize {
        self.agents.values().filter(|a| a.depth == depth).count()
    }

    /// Get total number of agents.
    pub fn get_total_count(&self) -> usize {
        self.agents.len()
    }

    /// Check if a parent agent can spawn more children (depth check).
    pub fn can_spawn(&self, parent_id: &str) -> bool {
        if let Some(parent) = self.agents.get(parent_id) {
            parent.depth + 1 < self.max_depth
        } else {
            false
        }
    }

    /// Cancel an agent and all its descendants.
    pub fn cancel_agent(&mut self, id: &str) -> Result<(), String> {
        let agent = self
            .agents
            .get(id)
            .ok_or_else(|| format!("Agent '{}' not found", id))?;
        let children = agent.children.clone();

        if let Some(agent) = self.agents.get_mut(id) {
            agent.status = RecursiveAgentStatus::Cancelled;
            agent.completed_at = Some(Utc::now());
        }

        // Recursively cancel children
        for child_id in &children {
            let _ = self.cancel_agent(child_id);
        }

        Ok(())
    }

    /// Render the agent tree as indented lines for display.
    pub fn render_tree(&self) -> Vec<String> {
        let roots = self.get_tree_roots();
        let mut lines = Vec::new();
        for root in &roots {
            render_node(root, &self.agents, 0, &mut lines);
        }
        lines
    }
}

/// Recursively render an agent node and its children with tree characters.
fn render_node(
    node: &AgentNode,
    agents: &HashMap<String, AgentNode>,
    depth: u8,
    lines: &mut Vec<String>,
) {
    let prefix = match depth {
        0 => String::new(),
        _ => {
            let mut p = "  ".repeat(depth as usize);
            p.push_str("├── ");
            p
        }
    };

    let status_icon = match node.status {
        RecursiveAgentStatus::Spawning => "⏳",
        RecursiveAgentStatus::Running => "🔄",
        RecursiveAgentStatus::Completed => "✅",
        RecursiveAgentStatus::Failed => "❌",
        RecursiveAgentStatus::Cancelled => "🚫",
        RecursiveAgentStatus::MaxDepthReached => "⛔",
    };

    let prompt_preview: String = node.prompt.chars().take(60).collect();
    lines.push(format!(
        "{}{} [depth {}] {} \"{}\"",
        prefix,
        status_icon,
        node.depth,
        &node.id[..8.min(node.id.len())],
        prompt_preview
    ));

    let children: Vec<&AgentNode> = agents
        .values()
        .filter(|a| a.parent_id.as_deref() == Some(&node.id))
        .collect();
    for child in &children {
        render_node(child, agents, depth + 1, lines);
    }
}

/// Generate a simple unique ID (first 8 chars of UUID-like string).
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}", t)
}

impl Default for RecursiveAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_top_level_agent() {
        let mut mgr = RecursiveAgentManager::new();
        let id = mgr.spawn_agent(None, "test prompt").unwrap();
        let agent = mgr.get_agent(&id).unwrap();
        assert_eq!(agent.depth, 0);
        assert!(agent.parent_id.is_none());
        assert_eq!(agent.status, RecursiveAgentStatus::Running);
    }

    #[test]
    fn spawn_child_agent() {
        let mut mgr = RecursiveAgentManager::new();
        let parent_id = mgr.spawn_agent(None, "parent").unwrap();
        let child_id = mgr.spawn_agent(Some(&parent_id), "child").unwrap();
        let child = mgr.get_agent(&child_id).unwrap();
        assert_eq!(child.depth, 1);
        assert_eq!(child.parent_id.as_deref(), Some(parent_id.as_str()));

        // Verify parent has child
        let children = mgr.get_children(&parent_id);
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn max_depth_enforced() {
        let mut mgr = RecursiveAgentManager::new();
        let mut ids = vec![mgr.spawn_agent(None, "root").unwrap()];
        for i in 1..MAX_AGENT_DEPTH {
            let child_id = mgr
                .spawn_agent(Some(ids.last().unwrap()), &format!("level {}", i))
                .unwrap();
            ids.push(child_id);
        }
        // This should fail — depth would be 5 which equals MAX_AGENT_DEPTH
        let result = mgr.spawn_agent(Some(ids.last().unwrap()), "too deep");
        assert!(result.is_err());
    }

    #[test]
    fn can_spawn_respects_depth() {
        let mut mgr = RecursiveAgentManager::new();
        let id = mgr.spawn_agent(None, "root").unwrap();
        assert!(mgr.can_spawn(&id));
    }

    #[test]
    fn cancel_propagates_to_children() {
        let mut mgr = RecursiveAgentManager::new();
        let parent_id = mgr.spawn_agent(None, "parent").unwrap();
        let child_id = mgr.spawn_agent(Some(&parent_id), "child").unwrap();
        mgr.cancel_agent(&parent_id).unwrap();
        assert_eq!(
            mgr.get_agent(&parent_id).unwrap().status,
            RecursiveAgentStatus::Cancelled
        );
        assert_eq!(
            mgr.get_agent(&child_id).unwrap().status,
            RecursiveAgentStatus::Cancelled
        );
    }

    #[test]
    fn render_tree_produces_output() {
        let mut mgr = RecursiveAgentManager::new();
        let id = mgr.spawn_agent(None, "root task").unwrap();
        mgr.spawn_agent(Some(&id), "child task").unwrap();
        let tree = mgr.render_tree();
        assert!(!tree.is_empty());
        assert!(tree[0].contains("root task"));
    }

    #[test]
    fn get_depth_count_works() {
        let mut mgr = RecursiveAgentManager::new();
        mgr.spawn_agent(None, "root1").unwrap();
        mgr.spawn_agent(None, "root2").unwrap();
        assert_eq!(mgr.get_depth_count(0), 2);
        assert_eq!(mgr.get_total_count(), 2);
    }
}
