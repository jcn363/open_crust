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

pub struct SessionManager {
    cache_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("open_crust/sessions");

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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_fork_session() {
        let mgr = SessionManager::new();
        let test_id = "test-original";
        let messages = vec![json!({"role": "user", "content": "hello"})];

        // Save original
        mgr.save_session(test_id, &messages).unwrap();

        // Fork it
        let forked = mgr.fork_session(test_id, Some("test-fork")).unwrap();

        assert_eq!(forked.id, "test-fork");
        assert_eq!(forked.messages.len(), 1);
        assert_eq!(
            forked.messages[0],
            json!({"role": "user", "content": "hello"})
        );

        // Cleanup
        let _ = mgr.delete_session(test_id);
        let _ = mgr.delete_session("test-fork");
    }

    #[test]
    fn test_fork_nonexistent() {
        let mgr = SessionManager::new();
        let result = mgr.fork_session("does-not-exist", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_fork_auto_name() {
        let mgr = SessionManager::new();
        let test_id = "test-original-2";
        let messages = vec![json!({"role": "user", "content": "test"})];

        // Save original
        mgr.save_session(test_id, &messages).unwrap();

        // Fork without providing name
        let forked = mgr.fork_session(test_id, None).unwrap();

        // Should contain original_id and "fork"
        assert!(forked.id.contains("test-original-2"));
        assert!(forked.id.contains("fork"));

        // Cleanup
        let _ = mgr.delete_session(test_id);
        let _ = mgr.delete_session(&forked.id);
    }

    #[test]
    fn test_fork_duplicate_name() {
        let mgr = SessionManager::new();
        let test_id = "test-original-3";
        let messages = vec![json!({"role": "user", "content": "test"})];

        // Save original
        mgr.save_session(test_id, &messages).unwrap();

        // Create a session with the name we want to use
        mgr.save_session("test-fork-dup", &messages).unwrap();

        // Fork with a name that already exists
        let forked = mgr.fork_session(test_id, Some("test-fork-dup")).unwrap();

        // Should have a different name (with timestamp appended)
        assert_ne!(forked.id, "test-fork-dup");
        assert!(forked.id.contains("test-fork-dup"));

        // Cleanup
        let _ = mgr.delete_session(test_id);
        let _ = mgr.delete_session("test-fork-dup");
        let _ = mgr.delete_session(&forked.id);
    }
}
