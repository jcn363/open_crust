use crate::app::App;
use chrono::Utc;
use serde_json::json;
use std::fs;

/// Serialize the current tab's conversation to a shareable JSON file.
/// Returns the file path on success.
pub fn share_conversation(app: &App) -> Option<String> {
    let tab = app.tabs.get(app.active_tab)?;
    if tab.messages.is_empty() {
        return None;
    }

    let share_data = json!({
        "version": 1,
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

    let share_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".config/opencrust/shares");
    if fs::create_dir_all(&share_dir).is_err() {
        return None;
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("share-{}.json", timestamp);
    let share_path = share_dir.join(&filename);

    let content = serde_json::to_string_pretty(&share_data).ok()?;
    fs::write(&share_path, content).ok()?;

    share_path.to_str().map(|s| s.to_string())
}
