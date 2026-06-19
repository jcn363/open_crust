//! Scheduled Tasks System for cron-like automation
//! Inspired by Warp Terminal's scheduled commands and Devin Desktop's scheduled chores

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Schedule types for task execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScheduleType {
    /// Run once at a specific time
    Once,
    /// Run at regular intervals
    Interval,
    /// Run daily at a specific time
    Daily,
    /// Run weekly on specific days
    Weekly,
    /// Run monthly on specific day
    Monthly,
    /// Run with custom cron expression
    Cron(String),
}

impl ScheduleType {
    /// Get icon for display
    pub fn icon(&self) -> &str {
        match self {
            ScheduleType::Once => "⏰",
            ScheduleType::Interval => "🔄",
            ScheduleType::Daily => "📅",
            ScheduleType::Weekly => "📆",
            ScheduleType::Monthly => "🗓️",
            ScheduleType::Cron(_) => "⚡",
        }
    }
}

/// A scheduled task with execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this task does
    pub description: String,
    /// Command or workflow to execute
    pub command: String,
    /// Schedule type
    pub schedule: ScheduleType,
    /// Whether this task is enabled
    pub enabled: bool,
    /// When the task was created
    pub created_at: SystemTime,
    /// When the task was last executed
    pub last_executed: Option<SystemTime>,
    /// When the task will next execute
    pub next_execution: Option<SystemTime>,
    /// Execution count
    pub execution_count: u32,
    /// Whether to notify on completion
    pub notify_on_completion: bool,
    /// Whether to notify on failure
    pub notify_on_failure: bool,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
    /// Space ID this task belongs to
    pub space_id: Option<String>,
    /// Agent ID to use for execution
    pub agent_id: Option<String>,
    /// Execution history
    pub history: Vec<ExecutionRecord>,
}

/// Record of a task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// When the execution started
    pub started_at: SystemTime,
    /// When the execution completed
    pub completed_at: Option<SystemTime>,
    /// Whether the execution was successful
    pub success: bool,
    /// Output from the execution
    pub output: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration of execution
    pub duration: Option<Duration>,
}

impl ScheduledTask {
    /// Create a new scheduled task
    pub fn new(name: String, description: String, command: String, schedule: ScheduleType) -> Self {
        let now = SystemTime::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            command,
            schedule,
            enabled: true,
            created_at: now,
            last_executed: None,
            next_execution: None,
            execution_count: 0,
            notify_on_completion: true,
            notify_on_failure: true,
            tags: Vec::new(),
            metadata: HashMap::new(),
            space_id: None,
            agent_id: None,
            history: Vec::new(),
        }
    }

    /// Calculate next execution time based on schedule
    pub fn calculate_next_execution(&self) -> Option<SystemTime> {
        let now = SystemTime::now();
        match &self.schedule {
            ScheduleType::Once => {
                if self.last_executed.is_some() {
                    None // Already executed
                } else {
                    Some(now) // Execute immediately
                }
            }
            ScheduleType::Interval => {
                // Default to 1 hour interval
                Some(now + Duration::from_secs(3600))
            }
            ScheduleType::Daily => {
                // Default to next day
                Some(now + Duration::from_secs(86400))
            }
            ScheduleType::Weekly => {
                // Default to next week
                Some(now + Duration::from_secs(604800))
            }
            ScheduleType::Monthly => {
                // Default to next month (30 days)
                Some(now + Duration::from_secs(2592000))
            }
            ScheduleType::Cron(_) => {
                // For cron expressions, we'd need a cron parser
                // For now, default to 1 hour
                Some(now + Duration::from_secs(3600))
            }
        }
    }

    /// Record execution start
    pub fn record_execution_start(&mut self) {
        self.last_executed = Some(SystemTime::now());
        self.execution_count += 1;
    }

    /// Record execution completion
    pub fn record_execution_completion(
        &mut self,
        success: bool,
        output: Option<String>,
        error: Option<String>,
    ) {
        let now = SystemTime::now();
        let duration = self
            .last_executed
            .map(|start| now.duration_since(start).unwrap_or_default());

        let record = ExecutionRecord {
            started_at: self.last_executed.unwrap_or(now),
            completed_at: Some(now),
            success,
            output,
            error,
            duration,
        };

        self.history.push(record);

        // Keep only last 100 execution records
        if self.history.len() > 100 {
            self.history.drain(0..self.history.len() - 100);
        }

        // Calculate next execution
        self.next_execution = self.calculate_next_execution();
    }

    /// Enable/disable task
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Toggle enabled status
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Add tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove tag
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Check if task is due for execution
    pub fn is_due(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(next) = self.next_execution {
            SystemTime::now() >= next
        } else {
            // No next execution scheduled
            false
        }
    }

    /// Get execution statistics
    pub fn execution_stats(&self) -> (u32, u32, f32) {
        let total = self.execution_count;
        let successful = self.history.iter().filter(|r| r.success).count() as u32;
        let success_rate = if total > 0 {
            (successful as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        (total, successful, success_rate)
    }
}

/// Manages scheduled tasks
#[derive(Debug, Default)]
pub struct TaskScheduler {
    /// All scheduled tasks indexed by ID
    tasks: HashMap<String, ScheduledTask>,
    /// Directory for saving scheduler state
    storage_dir: Option<PathBuf>,
    /// Whether the scheduler is running
    running: bool,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new() -> Self {
        Self::default()
    }

    /// Set storage directory
    pub fn with_storage(mut self, dir: PathBuf) -> Self {
        self.storage_dir = Some(dir);
        self
    }

    /// Create a new scheduled task
    pub fn create_task(
        &mut self,
        name: String,
        description: String,
        command: String,
        schedule: ScheduleType,
    ) -> &ScheduledTask {
        let mut task = ScheduledTask::new(name, description, command, schedule);
        task.next_execution = task.calculate_next_execution();
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        self.tasks.get(&id).unwrap()
    }

    /// Get a task by ID
    pub fn get_task(&self, id: &str) -> Option<&ScheduledTask> {
        self.tasks.get(id)
    }

    /// Get a mutable reference to a task
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut ScheduledTask> {
        self.tasks.get_mut(id)
    }

    /// List all tasks (optionally filtered)
    pub fn list_tasks(&self, enabled_only: bool) -> Vec<&ScheduledTask> {
        self.tasks
            .values()
            .filter(|t| !enabled_only || t.enabled)
            .collect()
    }

    /// Delete a task
    pub fn delete_task(&mut self, id: &str) -> bool {
        self.tasks.remove(id).is_some()
    }

    /// Search tasks
    pub fn search(&self, query: &str) -> Vec<&ScheduledTask> {
        let query_lower = query.to_lowercase();
        self.tasks
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower)
                    || t.description.to_lowercase().contains(&query_lower)
                    || t.tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get tasks due for execution
    pub fn due_tasks(&self) -> Vec<&ScheduledTask> {
        self.tasks.values().filter(|t| t.is_due()).collect()
    }

    /// Get task count
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Get enabled task count
    pub fn enabled_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.enabled).count()
    }

    /// Start the scheduler
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the scheduler
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if scheduler is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Save scheduler state to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(dir) = &self.storage_dir {
            std::fs::create_dir_all(dir)?;
            let json = serde_json::to_string_pretty(&self.tasks)?;
            std::fs::write(dir.join("scheduler.json"), json)?;
        }
        Ok(())
    }

    /// Load scheduler state from disk
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(dir) = &self.storage_dir {
            let path = dir.join("scheduler.json");
            if path.exists() {
                let json = std::fs::read_to_string(path)?;
                self.tasks = serde_json::from_str(&json)?;
            }
        }
        Ok(())
    }

    /// Import task from JSON string
    pub fn import_task(
        &mut self,
        json: &str,
    ) -> Result<&ScheduledTask, Box<dyn std::error::Error + Send + Sync>> {
        let task: ScheduledTask = serde_json::from_str(json)?;
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        Ok(self.tasks.get(&id).unwrap())
    }

    /// Export task to JSON string
    pub fn export_task(
        &self,
        id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(task) = self.tasks.get(id) {
            Ok(serde_json::to_string_pretty(task)?)
        } else {
            Err("Task not found".into())
        }
    }

    /// Create built-in scheduled tasks
    pub fn create_built_in_tasks(&mut self) {
        // Daily backup task
        let backup_task = ScheduledTask::new(
            "Daily Backup".to_string(),
            "Run daily backup of important files".to_string(),
            "tar -czf backup_$(date +%Y%m%d).tar.gz ~/Documents".to_string(),
            ScheduleType::Daily,
        );
        let id = backup_task.id.clone();
        self.tasks.insert(id, backup_task);

        // Weekly cleanup task
        let cleanup_task = ScheduledTask::new(
            "Weekly Cleanup".to_string(),
            "Clean up temporary files".to_string(),
            "find /tmp -type f -mtime +7 -delete".to_string(),
            ScheduleType::Weekly,
        );
        let id = cleanup_task.id.clone();
        self.tasks.insert(id, cleanup_task);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_task_creation() {
        let task = ScheduledTask::new(
            "Test Task".to_string(),
            "A test task".to_string(),
            "echo test".to_string(),
            ScheduleType::Daily,
        );
        assert_eq!(task.name, "Test Task");
        assert!(task.enabled);
        assert_eq!(task.execution_count, 0);
    }

    #[test]
    fn test_task_scheduler() {
        let mut scheduler = TaskScheduler::new();
        let task_id = {
            let task = scheduler.create_task(
                "Test".to_string(),
                "Test task".to_string(),
                "echo test".to_string(),
                ScheduleType::Daily,
            );
            task.id.clone()
        };
        assert_eq!(scheduler.task_count(), 1);
        assert!(scheduler.get_task(&task_id).is_some());
    }

    #[test]
    fn test_task_execution() {
        let mut task = ScheduledTask::new(
            "Test".to_string(),
            "Test".to_string(),
            "echo test".to_string(),
            ScheduleType::Daily,
        );

        task.record_execution_start();
        task.record_execution_completion(true, Some("output".to_string()), None);

        assert_eq!(task.execution_count, 1);
        assert!(!task.history.is_empty());
    }
}
