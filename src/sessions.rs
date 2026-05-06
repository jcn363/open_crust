use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

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
        
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");
        }
        
        Self { cache_dir }
    }
    
    pub fn save_session(&self, id: &str, messages: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
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
            entries.flatten()
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
}
