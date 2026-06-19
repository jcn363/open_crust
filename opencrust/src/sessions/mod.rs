//! Session persistence and management
//!
//! Saves, loads, lists, and deletes chat sessions as JSON files. Supports
//! forking a session into a new named variant for experimentation, tracking
//! message history across restarts.
//! Supports checkpoints (snapshots) for session rollback.

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

/// A checkpoint (snapshot) of a session at a point in time.
///
/// Used for rollback functionality — allows restoring a session
/// to a previous state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub name: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Value>,
}

/// Manages session persistence — save, load, list, delete, fork, checkpoint
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
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
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

    /// Get the checkpoint directory for a session
    fn checkpoint_dir(&self, session_id: &str) -> PathBuf {
        self.cache_dir.join("checkpoints").join(session_id)
    }

    /// Create a checkpoint (snapshot) of a session
    pub fn create_checkpoint(
        &self,
        session_id: &str,
        name: Option<&str>,
    ) -> Result<Checkpoint, Box<dyn std::error::Error>> {
        // Load the session to checkpoint
        let session = self.load_session(session_id)?;

        // Generate checkpoint name if not provided
        let checkpoint_name = name
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("checkpoint-{}", Utc::now().timestamp()));

        let checkpoint = Checkpoint {
            name: checkpoint_name.clone(),
            session_id: session_id.to_string(),
            timestamp: Utc::now(),
            messages: session.messages.clone(),
        };

        // Ensure checkpoint directory exists
        let cp_dir = self.checkpoint_dir(session_id);
        if !cp_dir.exists() {
            fs::create_dir_all(&cp_dir)?;
        }

        // Save checkpoint
        let cp_path = cp_dir.join(format!("{}.json", checkpoint_name));
        let content = serde_json::to_string_pretty(&checkpoint)?;
        fs::write(cp_path, content)?;

        Ok(checkpoint)
    }

    /// List all checkpoints for a session
    pub fn list_checkpoints(&self, session_id: &str) -> Vec<Checkpoint> {
        let cp_dir = self.checkpoint_dir(session_id);
        if let Ok(entries) = fs::read_dir(&cp_dir) {
            entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .filter_map(|e| {
                    let content = fs::read_to_string(e.path()).ok()?;
                    serde_json::from_str::<Checkpoint>(&content).ok()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Restore a session from a checkpoint
    pub fn restore_checkpoint(
        &self,
        session_id: &str,
        checkpoint_name: &str,
    ) -> Result<Session, Box<dyn std::error::Error>> {
        let cp_dir = self.checkpoint_dir(session_id);
        let cp_path = cp_dir.join(format!("{}.json", checkpoint_name));

        if !cp_path.exists() {
            return Err(format!("Checkpoint not found: {}", checkpoint_name).into());
        }

        let content = fs::read_to_string(cp_path)?;
        let checkpoint: Checkpoint = serde_json::from_str(&content)?;

        // Create a new session with the checkpoint's messages
        let restored_session = Session {
            id: session_id.to_string(),
            timestamp: Utc::now(),
            messages: checkpoint.messages.clone(),
        };

        // Save the restored session (overwrites current session)
        let session_path = self.cache_dir.join(format!("{}.json", session_id));
        let content = serde_json::to_string_pretty(&restored_session)?;
        fs::write(session_path, content)?;

        Ok(restored_session)
    }

    /// Delete a checkpoint
    pub fn delete_checkpoint(
        &self,
        session_id: &str,
        checkpoint_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cp_dir = self.checkpoint_dir(session_id);
        let cp_path = cp_dir.join(format!("{}.json", checkpoint_name));

        if cp_path.exists() {
            fs::remove_file(cp_path)?;
            Ok(())
        } else {
            Err(format!("Checkpoint not found: {}", checkpoint_name).into())
        }
    }
}

#[cfg(test)]
mod tests;
