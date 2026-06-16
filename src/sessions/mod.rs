//! Session persistence and management
//!
//! Saves, loads, lists, and deletes chat sessions as JSON files. Supports
//! forking a session into a new named variant for experimentation, tracking
//! message history across restarts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Represents a persisted session with message history
///
/// Used for serializing/deserializing session state to disk.
/// Now actively used by CLI commands (list, show, delete).
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Value>,
}

/// Manages session persistence — save, load, list, delete, fork
///
/// Stores chat sessions as JSON files in the user's cache directory.
/// Supports forking sessions for experimentation branches.
pub struct SessionManager {
    cache_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencrust/sessions");

        if !cache_dir.exists()
            && let Err(e) = fs::create_dir_all(&cache_dir)
        {
            eprintln!("Warning: Failed to create session cache dir: {}", e);
        }

        Self { cache_dir }
    }

    pub fn save_session(
        &self,
        id: &str,
        messages: &[Value],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let session = Session {
            id: id.to_string(),
            timestamp: Utc::now(),
            messages: messages.to_vec(),
        };

        let session_path = self.cache_dir.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&session)?;
        fs::write(session_path, content)?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<Session> {
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .filter_map(|e| {
                    let content = fs::read_to_string(e.path()).ok()?;
                    serde_json::from_str::<Session>(&content).ok()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn load_session(&self, id: &str) -> Result<Session, Box<dyn std::error::Error>> {
        let session_path = self.cache_dir.join(format!("{}.json", id));
        let content = fs::read_to_string(session_path)?;
        let session: Session = serde_json::from_str(&content)?;
        Ok(session)
    }

    pub fn delete_session(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let session_path = self.cache_dir.join(format!("{}.json", id));
        if session_path.exists() {
            fs::remove_file(session_path)?;
            Ok(())
        } else {
            Err(format!("Session not found: {}", id).into())
        }
    }

    pub fn fork_session(
        &self,
        original_id: &str,
        new_id: Option<&str>,
    ) -> Result<Session, Box<dyn std::error::Error>> {
        // 1. Load original session
        let original = self.load_session(original_id)?;

        // 2. Determine new session ID
        let final_id = match new_id {
            Some(id) => id.to_string(),
            None => format!("{}-fork-{}", original_id, chrono::Utc::now().timestamp()),
        };

        // 3. Check if new_id already exists (auto-disambiguate)
        let mut final_id = final_id;
        let session_path = self.cache_dir.join(format!("{}.json", final_id));
        if session_path.exists() {
            final_id = format!("{}-{}", final_id, chrono::Utc::now().timestamp());
        }

        // 4. Create new session with copied messages
        let new_session = Session {
            id: final_id.clone(),
            timestamp: chrono::Utc::now(),
            messages: original.messages.clone(),
        };

        // 5. Save to disk
        let content = serde_json::to_string_pretty(&new_session)?;
        let session_path = self.cache_dir.join(format!("{}.json", final_id));
        fs::write(session_path, content)?;

        Ok(new_session)
    }
}

#[cfg(test)]
mod tests;
