//! Spaces/Projects management for Mission Control
//! Inspired by Devin Desktop's Spaces feature for grouping agents by project

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// A Space represents a project or workspace that groups related agents and tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Unique identifier for the space
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// When the space was created
    pub created_at: SystemTime,
    /// Last activity in this space
    pub last_activity: SystemTime,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Whether this space is archived
    pub archived: bool,
    /// Agent IDs associated with this space
    pub agent_ids: Vec<String>,
    /// Task IDs associated with this space
    pub task_ids: Vec<uuid::Uuid>,
    /// Shared context for agents in this space
    pub shared_context: HashMap<String, String>,
}

impl Space {
    /// Create a new space with the given name
    pub fn new(name: String) -> Self {
        let now = SystemTime::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            created_at: now,
            last_activity: now,
            tags: Vec::new(),
            archived: false,
            agent_ids: Vec::new(),
            task_ids: Vec::new(),
            shared_context: HashMap::new(),
        }
    }

    /// Add an agent to this space
    pub fn add_agent(&mut self, agent_id: String) {
        if !self.agent_ids.contains(&agent_id) {
            self.agent_ids.push(agent_id);
            self.last_activity = SystemTime::now();
        }
    }

    /// Remove an agent from this space
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.agent_ids.retain(|id| id != agent_id);
        self.last_activity = SystemTime::now();
    }

    /// Add a task to this space
    pub fn add_task(&mut self, task_id: uuid::Uuid) {
        if !self.task_ids.contains(&task_id) {
            self.task_ids.push(task_id);
            self.last_activity = SystemTime::now();
        }
    }

    /// Remove a task from this space
    pub fn remove_task(&mut self, task_id: &uuid::Uuid) {
        self.task_ids.retain(|id| id != task_id);
        self.last_activity = SystemTime::now();
    }

    /// Set shared context value
    pub fn set_context(&mut self, key: String, value: String) {
        self.shared_context.insert(key, value);
        self.last_activity = SystemTime::now();
    }

    /// Get shared context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.shared_context.get(key)
    }

    /// Archive this space
    pub fn archive(&mut self) {
        self.archived = true;
        self.last_activity = SystemTime::now();
    }

    /// Unarchive this space
    pub fn unarchive(&mut self) {
        self.archived = false;
        self.last_activity = SystemTime::now();
    }

    /// Check if space is active (not archived and has recent activity)
    pub fn is_active(&self) -> bool {
        !self.archived
    }

    /// Get age of space
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or_default()
    }

    /// Get time since last activity
    pub fn last_activity_age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.last_activity)
            .unwrap_or_default()
    }
}

/// Manages multiple spaces/projects
#[derive(Debug, Default)]
pub struct SpaceManager {
    /// All spaces indexed by ID
    spaces: HashMap<String, Space>,
    /// Currently active space ID
    active_space_id: Option<String>,
    /// Path to spaces persistence file
    persistence_path: Option<PathBuf>,
}

impl SpaceManager {
    /// Create a new space manager
    pub fn new() -> Self {
        Self {
            spaces: HashMap::new(),
            active_space_id: None,
            persistence_path: None,
        }
    }

    /// Set persistence path for saving/loading spaces
    pub fn with_persistence(mut self, path: PathBuf) -> Self {
        self.persistence_path = Some(path);
        self
    }

    /// Create a new space
    pub fn create_space(&mut self, name: String) -> &Space {
        let space = Space::new(name);
        let id = space.id.clone();
        self.spaces.insert(id.clone(), space);
        self.spaces.get(&id).unwrap()
    }

    /// Get a space by ID
    pub fn get_space(&self, id: &str) -> Option<&Space> {
        self.spaces.get(id)
    }

    /// Get a mutable reference to a space
    pub fn get_space_mut(&mut self, id: &str) -> Option<&mut Space> {
        self.spaces.get_mut(id)
    }

    /// List all spaces (optionally filtered)
    pub fn list_spaces(&self, include_archived: bool) -> Vec<&Space> {
        self.spaces
            .values()
            .filter(|s| include_archived || !s.archived)
            .collect()
    }

    /// Set the active space
    pub fn set_active_space(&mut self, space_id: Option<String>) {
        self.active_space_id = space_id;
    }

    /// Get the active space
    pub fn get_active_space(&self) -> Option<&Space> {
        self.active_space_id
            .as_ref()
            .and_then(|id| self.spaces.get(id))
    }

    /// Get the active space ID
    pub fn active_space_id(&self) -> Option<&str> {
        self.active_space_id.as_deref()
    }

    /// Delete a space
    pub fn delete_space(&mut self, id: &str) -> bool {
        if self.active_space_id.as_deref() == Some(id) {
            self.active_space_id = None;
        }
        self.spaces.remove(id).is_some()
    }

    /// Get space count
    pub fn space_count(&self) -> usize {
        self.spaces.len()
    }

    /// Get active (non-archived) space count
    pub fn active_space_count(&self) -> usize {
        self.spaces.values().filter(|s| !s.archived).count()
    }

    /// Save spaces to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(path) = &self.persistence_path {
            let json = serde_json::to_string_pretty(&self.spaces)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }

    /// Load spaces from disk
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(path) = &self.persistence_path {
            if path.exists() {
                let json = std::fs::read_to_string(path)?;
                self.spaces = serde_json::from_str(&json)?;
            }
        }
        Ok(())
    }

    /// Search spaces by name or tags
    pub fn search(&self, query: &str) -> Vec<&Space> {
        let query_lower = query.to_lowercase();
        self.spaces
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || s.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get recently active spaces
    pub fn recently_active(&self, limit: usize) -> Vec<&Space> {
        let mut spaces: Vec<&Space> = self.spaces.values().collect();
        spaces.sort_by_key(|b| std::cmp::Reverse(b.last_activity));
        spaces.into_iter().take(limit).collect()
    }

    /// Get spaces with most agents
    pub fn most_agents(&self, limit: usize) -> Vec<&Space> {
        let mut spaces: Vec<&Space> = self.spaces.values().collect();
        spaces.sort_by_key(|b| std::cmp::Reverse(b.agent_ids.len()));
        spaces.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_creation() {
        let space = Space::new("Test Project".to_string());
        assert_eq!(space.name, "Test Project");
        assert!(!space.archived);
        assert!(space.agent_ids.is_empty());
        assert!(space.task_ids.is_empty());
    }

    #[test]
    fn test_space_agent_management() {
        let mut space = Space::new("Test".to_string());
        space.add_agent("agent1".to_string());
        space.add_agent("agent2".to_string());
        assert_eq!(space.agent_ids.len(), 2);

        space.remove_agent("agent1");
        assert_eq!(space.agent_ids.len(), 1);
        assert_eq!(space.agent_ids[0], "agent2");
    }

    #[test]
    fn test_space_task_management() {
        let mut space = Space::new("Test".to_string());
        let task_id = uuid::Uuid::new_v4();
        space.add_task(task_id);
        assert_eq!(space.task_ids.len(), 1);

        space.remove_task(&task_id);
        assert!(space.task_ids.is_empty());
    }

    #[test]
    fn test_space_manager() {
        let mut manager = SpaceManager::new();
        let space = manager.create_space("Project A".to_string());
        let space_id = space.id.clone();

        assert_eq!(manager.space_count(), 1);
        assert!(manager.get_space(&space_id).is_some());

        manager.set_active_space(Some(space_id.clone()));
        assert_eq!(manager.active_space_id(), Some(space_id.as_str()));
    }
}
