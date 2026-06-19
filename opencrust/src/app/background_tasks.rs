//! Background task spawning and notification handling

use crate::app::App;

impl App {
    /// Spawn a background task with the given prompt
    pub fn spawn_background_task(&mut self, prompt: String) {
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let task = crate::app::BackgroundTask {
            id: task_id.clone(),
            prompt: prompt.clone(),
            status: crate::app::TaskStatus::Running,
            result: None,
            started_at: now,
        };

        self.background_tasks.push(task);

        if let Some(tx) = &self.background_task_tx {
            let tx_clone = tx.clone();
            let llm = self.llm_client.clone();

            tokio::spawn(async move {
                let result = llm.query_simple(&prompt, None).await;
                let response = match result {
                    Ok(content) => format!("[TASK_COMPLETE]{}::{}", task_id, content),
                    Err(e) => format!("[TASK_FAILED]{}::{}", task_id, e),
                };
                let _ = tx_clone.send(response).await;
            });
        }
    }

    /// Handle background task notifications.
    /// Returns the tab index that was modified, if any.
    pub fn handle_background_notification(&mut self, notification: &str) -> Option<usize> {
        if notification.starts_with("[TASK_COMPLETE]") {
            let parts: Vec<&str> = notification
                .strip_prefix("[TASK_COMPLETE]")
                .map(|s| s.splitn(2, "::").collect())
                .unwrap_or_default();
            if parts.len() == 2 {
                let task_id = parts[0].to_string();
                let result = parts[1].to_string();
                if let Some(task) = self.background_tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = crate::app::TaskStatus::Completed;
                    task.result = Some(result.clone());
                }
                let tab_idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
                self.tabs[tab_idx]
                    .messages
                    .push(crate::app::Message::new(format!(
                        "Task {} completed: {}",
                        task_id, result
                    )));
                return Some(tab_idx);
            }
        } else if notification.starts_with("[TASK_FAILED]") {
            let parts: Vec<&str> = notification
                .strip_prefix("[TASK_FAILED]")
                .map(|s| s.splitn(2, "::").collect())
                .unwrap_or_default();
            if parts.len() == 2 {
                let task_id = parts[0].to_string();
                let error = parts[1].to_string();
                if let Some(task) = self.background_tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = crate::app::TaskStatus::Failed;
                    task.result = Some(error.clone());
                }
                let tab_idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
                self.tabs[tab_idx]
                    .messages
                    .push(crate::app::Message::new(format!(
                        "Task {} failed: {}",
                        task_id, error
                    )));
                return Some(tab_idx);
            }
        }
        None
    }
}
