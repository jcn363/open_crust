//! Notification tool handler

use serde_json::Value;

use crate::desktop::notifications;

/// Execute a notification tool by name. Returns Some(result) if handled.
pub fn execute_notify_tool(name: &str, args: &Value) -> Option<String> {
    match name {
        "notify" => {
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Notification");
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let urgency = args
                .get("urgency")
                .and_then(|v| v.as_str())
                .unwrap_or("normal");

            let notif = notifications::Notification::new(title, body)
                .with_urgency(notifications::NotificationUrgency::from_name(urgency));

            Some(match notifications::send_notification_smart(&notif) {
                Ok(_) => format!("Notification sent: {} - {}", title, body),
                Err(e) => format!("Failed to send notification: {}", e),
            })
        }
        _ => None,
    }
}
