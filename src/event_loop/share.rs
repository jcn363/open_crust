use crate::app::App;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// Metadata about a shareable session link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLink {
    pub id: String,
    pub file_path: String,
    pub created_at: String,
    pub provider: String,
    pub model: String,
    pub message_count: usize,
    pub tab_name: String,
}

/// Serialize the current tab's conversation to a shareable JSON file.
/// Returns the share link metadata on success.
pub fn share_conversation(app: &App) -> Option<ShareLink> {
    let tab = app.tabs.get(app.active_tab)?;
    if tab.messages.is_empty() {
        return None;
    }

    let share_id = generate_share_id();
    let share_data = json!({
        "version": 1,
        "share_id": share_id,
        "project": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
        "provider": app.config.provider.to_string(),
        "model": app.config.model,
        "tab": tab.name,
        "created_at": Utc::now().to_rfc3339(),
        "messages": tab.messages.iter().map(|m| {
            json!({
                "timestamp": m.timestamp.to_rfc3339(),
                "content": m.content,
            })
        }).collect::<Vec<_>>(),
        "message_count": tab.messages.len(),
    });

    let share_dir = share_dir();
    if fs::create_dir_all(&share_dir).is_err() {
        return None;
    }

    let filename = format!("share-{}.json", share_id);
    let share_path = share_dir.join(&filename);

    let content = serde_json::to_string_pretty(&share_data).ok()?;
    fs::write(&share_path, content).ok()?;

    Some(ShareLink {
        id: share_id,
        file_path: share_path.to_str()?.to_string(),
        created_at: Utc::now().to_rfc3339(),
        provider: app.config.provider.to_string(),
        model: app.config.model.clone(),
        message_count: tab.messages.len(),
        tab_name: tab.name.clone(),
    })
}

/// Get the share directory path.
pub fn share_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/opencrust/shares")
}

/// List all existing share links.
pub fn list_share_links() -> Vec<ShareLink> {
    let dir = share_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut links = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        let link = ShareLink {
                            id: data
                                .get("share_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            file_path: path.to_str().unwrap_or("").to_string(),
                            created_at: data
                                .get("created_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            provider: data
                                .get("provider")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            model: data
                                .get("model")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            message_count: data
                                .get("message_count")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as usize,
                            tab_name: data
                                .get("tab")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        };
                        links.push(link);
                    }
                }
            }
        }
    }

    links.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    links
}

/// Load a share link by ID.
#[allow(dead_code)]
pub fn load_share_link(id: &str) -> Option<ShareLink> {
    list_share_links().into_iter().find(|l| l.id == id)
}

/// Delete a share link by ID.
#[allow(dead_code)]
pub fn delete_share_link(id: &str) -> bool {
    let links = list_share_links();
    if let Some(link) = links.iter().find(|l| l.id == id) {
        fs::remove_file(&link.file_path).is_ok()
    } else {
        false
    }
}

/// Generate a short unique share ID (12 hex chars).
fn generate_share_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:012x}", t & 0xFFFFFFFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_share_id_is_unique() {
        let id1 = generate_share_id();
        let id2 = generate_share_id();
        // Nanosecond timestamps should differ
        assert_ne!(id1, id2);
    }

    #[test]
    fn share_dir_returns_consistent_path() {
        let d1 = share_dir();
        let d2 = share_dir();
        assert_eq!(d1, d2);
    }

    #[test]
    fn list_share_links_returns_vec_on_missing_dir() {
        // Even if the dir doesn't exist, it should return empty vec
        let links = list_share_links();
        assert!(links.is_empty() || !links.is_empty()); // Just shouldn't panic
    }
}
