use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Represents a persisted session with message history
///
/// Used for serializing/deserializing session state to disk.
/// Instances are created in `_save_session()` and support `pub` visibility
/// to enable external access patterns in future API expansions.
/// Marked as `#[allow(dead_code)]` because it's only used internally via serde
/// but exposed publicly for future CLI/API use.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Value>,
}

pub struct SessionManager {
    _cache_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("open_crust/sessions");
        
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");
        }

        Self { _cache_dir: cache_dir }
    }

    pub fn _save_session(&self, id: &str, messages: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
        let session = Session {
            id: id.to_string(),
            timestamp: Utc::now(),
            messages: messages.to_vec(),
        };

        let session_path = self._cache_dir.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&session)?;
        fs::write(session_path, content)?;
        Ok(())
    }

    pub fn _list_sessions(&self) -> Vec<String> {
        if let Ok(entries) = fs::read_dir(&self._cache_dir) {
            entries.flatten()
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
                .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().to_string()))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn _load_session(&self, id: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let session_path = self._cache_dir.join(format!("{}.json", id));
        let content = fs::read_to_string(session_path)?;
        let session: Session = serde_json::from_str(&content)?;
        Ok(session.messages)
    }
}
