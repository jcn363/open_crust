//! macOS notification backend — osascript (AppleScript)

use std::process::Command;

use super::linux::simple_hash;
use super::mod_types::{Notification, NotificationDaemon, NotificationUrgency};

/// Send a notification using macOS `osascript` (AppleScript).
pub fn send_notification_macos(notification: &Notification) -> Result<u32, String> {
    let mut script = String::from("display notification \"");
    let body = notification.body.replace('"', "\\\"");
    script.push_str(&body);
    script.push('"');

    if !notification.title.is_empty() {
        let title = notification.title.replace('"', "\\\"");
        script.push_str(" with title \"");
        script.push_str(&title);
        script.push('"');
    }

    if notification.options.urgency == NotificationUrgency::Critical {
        script.push_str(" subtitle \"Urgent\"");
    }

    let output = Command::new("osascript").arg("-e").arg(&script).output();

    match output {
        Ok(output) if output.status.success() => {
            let id = simple_hash(&format!("{}{}", notification.title, notification.body));
            Ok(id)
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("osascript failed: {}", error))
        }
        Err(e) => Err(format!("Failed to run osascript: {}", e)),
    }
}

/// Check if `osascript` is available on macOS.
pub fn is_macos_notifications_available() -> bool {
    Command::new("which")
        .arg("osascript")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check notification daemon availability on macOS.
pub fn check_notification_daemon_macos() -> NotificationDaemon {
    let has_osascript = is_macos_notifications_available();
    NotificationDaemon {
        available: has_osascript,
        name: if has_osascript {
            "osascript".to_string()
        } else {
            "none".to_string()
        },
        supports_actions: false,
        supports_inline_reply: false,
    }
}
