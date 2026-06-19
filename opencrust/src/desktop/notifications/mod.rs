//! Linux Mint Cinnamon system notifications
//!
//! Provides system notification integration for Linux Mint Cinnamon.
//! Uses notify-send as primary method, with DBus fallback for rich notifications.

use std::time::Duration;

pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod mod_types;

// Re-export types for backward compatibility.
pub use mod_types::{Notification, NotificationDaemon, NotificationUrgency};

// ── Public API ──

/// Check if notification daemon is available.
pub fn check_notification_daemon() -> NotificationDaemon {
    #[cfg(target_os = "macos")]
    {
        return macos::check_notification_daemon_macos();
    }
    #[cfg(not(target_os = "macos"))]
    {
        linux::check_notification_daemon_linux()
    }
}

/// Check if notifications require a running daemon
pub fn is_notification_available() -> bool {
    check_notification_daemon().available
}

/// Send a notification using the best available backend.
///
/// Platform dispatch:
/// - **macOS**: uses `osascript` (AppleScript `display notification`)
/// - **Linux/Unix**: tries DBus for rich features, falls back to notify-send
///
/// Returns the notification ID for use with `close_notification`.
pub fn send_notification_smart(notification: &Notification) -> Result<u32, String> {
    #[cfg(target_os = "macos")]
    {
        if macos::is_macos_notifications_available() {
            return macos::send_notification_macos(notification);
        }
    }

    let app_name = "OpenCrust";

    // Try DBus first for richer features
    if let Ok(id) = linux::send_notification_dbus(app_name, notification) {
        return Ok(id);
    }

    // Fall back to notify-send if DBus fails
    linux::send_notification(notification).map(|_| 0)
}

/// Show a transient notification with auto-dismiss
pub fn notify_timed(
    title: impl Into<String>,
    body: impl Into<String>,
    duration: Duration,
) -> Result<(), String> {
    let seconds = duration.as_secs() as u32;
    let notification = Notification::new(title, body).with_expire_timeout(seconds);
    send_notification_smart(&notification).map(|_| ())
}

/// Show an error notification
pub fn notify_error(title: impl Into<String>, body: impl Into<String>) -> Result<(), String> {
    let notification = Notification::new(title, body).with_urgency(NotificationUrgency::Critical);
    send_notification_smart(&notification).map(|_| ())
}

/// Show a success notification
pub fn notify_success(title: impl Into<String>, body: impl Into<String>) -> Result<(), String> {
    let notification = Notification::new(title, body)
        .with_urgency(NotificationUrgency::Low)
        .with_icon("dialog-information");
    send_notification_smart(&notification).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notif = Notification::new("Test", "Hello world");
        assert_eq!(notif.title, "Test");
        assert_eq!(notif.body, "Hello world");
    }

    #[test]
    fn test_notification_options() {
        let notif = Notification::new("Test", "Body")
            .with_urgency(NotificationUrgency::Critical)
            .with_expire_timeout(5)
            .with_icon("utilities-terminal");

        assert_eq!(notif.options.urgency, NotificationUrgency::Critical);
        assert_eq!(notif.options.expire_timeout, 5);
        assert_eq!(notif.options.icon, Some("utilities-terminal".to_string()));
    }

    #[test]
    fn test_urgency() {
        assert_eq!(NotificationUrgency::Low.as_arg(), "low");
        assert_eq!(NotificationUrgency::Critical.as_arg(), "critical");
    }
}
