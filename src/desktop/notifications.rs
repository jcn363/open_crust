//! Linux Mint Cinnamon system notifications
//!
//! Provides system notification integration for Linux Mint Cinnamon.
//! Uses notify-send as primary method, with DBus fallback for rich notifications.

use std::process::Command;
use std::time::Duration;

/// Notification urgency level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

impl NotificationUrgency {
    /// Convert to notify-send argument
    pub fn as_arg(&self) -> &str {
        match self {
            NotificationUrgency::Low => "low",
            NotificationUrgency::Normal => "normal",
            NotificationUrgency::Critical => "critical",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => NotificationUrgency::Low,
            "critical" => NotificationUrgency::Critical,
            _ => NotificationUrgency::Normal,
        }
    }
}

/// Notification category for organization
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NotificationCategory {
    #[default]
    General,
    #[expect(dead_code, reason = "NotificationCategory variant")]
    Message,
    #[expect(dead_code, reason = "NotificationCategory variant")]
    Email,
    #[expect(dead_code, reason = "NotificationCategory variant")]
    Alarm,
    #[expect(dead_code, reason = "NotificationCategory variant")]
    Transfer,
}

#[allow(dead_code)]
impl NotificationCategory {
    /// Get DBus category string
    pub fn to_dbus(&self) -> &str {
        match self {
            NotificationCategory::General => "general",
            NotificationCategory::Message => "im",
            NotificationCategory::Email => "email",
            NotificationCategory::Alarm => "alarm",
            NotificationCategory::Transfer => "transfer",
        }
    }
}

/// Notification options
#[derive(Debug, Clone, Default)]
pub struct NotificationOptions {
    /// Urgency level
    pub urgency: NotificationUrgency,
    /// Expire timeout in seconds (0 = use default)
    pub expire_timeout: u32,
    /// Icon to display
    pub icon: Option<String>,
    /// Category
    pub category: NotificationCategory,
    /// Action to invoke on click
    #[expect(dead_code, reason = "click action for notification")]
    pub action: Option<String>,
}

/// A notification instance
#[derive(Debug, Clone)]
pub struct Notification {
    /// Notification title
    pub title: String,
    /// Notification body/message
    pub body: String,
    /// Optional notification options
    pub options: NotificationOptions,
}

#[expect(dead_code, reason = "notification builder API")]
impl Notification {
    /// Create a new notification
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            options: NotificationOptions::default(),
        }
    }

    /// Set urgency level
    pub fn with_urgency(mut self, urgency: NotificationUrgency) -> Self {
        self.options.urgency = urgency;
        self
    }

    /// Set expire timeout in seconds
    pub fn with_expire_timeout(mut self, seconds: u32) -> Self {
        self.options.expire_timeout = seconds;
        self
    }

    /// Set custom icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.options.icon = Some(icon.into());
        self
    }

    /// Set category
    pub fn with_category(mut self, category: NotificationCategory) -> Self {
        self.options.category = category;
        self
    }
}

/// Notification daemon status
#[derive(Debug, Clone, Default)]
#[expect(dead_code, reason = "daemon capability inspection")]
pub struct NotificationDaemon {
    /// Whether notifications are available
    pub available: bool,
    /// Daemon name (e.g., "notify-osd", "mintdesktop")
    pub name: String,
    /// Supports actions (clickable buttons)
    pub supports_actions: bool,
    /// Supports inline replies
    pub supports_inline_reply: bool,
}

/// Check if notification daemon is available
#[allow(dead_code)]
pub fn check_notification_daemon() -> NotificationDaemon {
    // Try to get info from dbus-send
    let output = Command::new("dbus-send")
        .args([
            "--session",
            "--dest=org.freedesktop.Notifications",
            "--type=method_call",
            "--print-reply",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications.GetServerInformation",
        ])
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let info = String::from_utf8_lossy(&output.stdout);
        // Parse response like: string "notify-osd" string "MATE" string "1.0" string "1.0"
        let parts: Vec<&str> = info.lines().filter(|l| !l.is_empty()).collect();
        if parts.len() >= 4 {
            return NotificationDaemon {
                available: true,
                name: parts[0].trim_matches('"').to_string(),
                supports_actions: false,
                supports_inline_reply: false,
            };
        }
    };

    // Fallback: check if notify-send exists
    let has_notify_send = Command::new("which")
        .arg("notify-send")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    NotificationDaemon {
        available: has_notify_send,
        name: if has_notify_send {
            "notify-send".to_string()
        } else {
            "none".to_string()
        },
        supports_actions: false,
        supports_inline_reply: false,
    }
}

/// Check if notifications require a running daemon
#[expect(dead_code, reason = "capability check API")]
pub fn is_notification_available() -> bool {
    check_notification_daemon().available
}

/// Send a notification using notify-send (most compatible method)
pub fn send_notification(notification: &Notification) -> Result<(), String> {
    let mut args = vec![
        "-u".to_string(),
        notification.options.urgency.as_arg().to_string(),
    ];

    // Add expire timeout if specified
    if notification.options.expire_timeout > 0 {
        args.push("-t".to_string());
        args.push((notification.options.expire_timeout * 1000).to_string()); // Convert to ms
    }

    // Add icon if specified
    if let Some(ref icon) = notification.options.icon {
        args.push("-i".to_string());
        args.push(icon.clone());
    }

    // Add category if specified
    if notification.options.category != NotificationCategory::General {
        args.push("-c".to_string());
        args.push(notification.options.category.to_dbus().to_string());
    }

    // Title
    args.push(notification.title.clone());
    // Body
    args.push(notification.body.clone());

    let output = Command::new("notify-send").args(&args).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("notify-send failed: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to run notify-send: {}", e)),
    }
}

/// Send a notification via DBus (enables rich notifications with actions)
///
/// Uses dbus-send to invoke the Freedesktop Notifications DBus interface.
/// This enables richer notifications with action buttons and persistence.
pub fn send_notification_dbus(app_name: &str, notification: &Notification) -> Result<u32, String> {
    let urgency: u32 = match notification.options.urgency {
        NotificationUrgency::Low => 0,
        NotificationUrgency::Normal => 1,
        NotificationUrgency::Critical => 2,
    };

    let expire_timeout = if notification.options.expire_timeout > 0 {
        notification.options.expire_timeout as i32 * 1000 // Convert to ms
    } else {
        -1 // Default
    };

    let icon = notification
        .options
        .icon
        .clone()
        .unwrap_or_else(|| "dialog-information".to_string());

    // Build DBus method call
    let output = Command::new("dbus-send")
        .args([
            "--session",
            "--dest=org.freedesktop.Notifications",
            "--type=method_call",
            "--print-reply",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications.Notify",
        ])
        .arg(format!("string:{}", app_name))
        .arg("uint32:0") // Replace ID (0 = new)
        .arg(format!("string:{}", icon))
        .arg(format!("string:{}", notification.title))
        .arg(format!("string:{}", notification.body))
        .arg("array:string:[]") // Actions (empty for no buttons)
        .arg(format!("dict:string:variant:urgency,byte:{}", urgency)) // Hints: include urgency
        .arg(format!("int32:{}", expire_timeout))
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                // Parse the notification ID from response
                let response = String::from_utf8_lossy(&output.stdout);
                for line in response.lines() {
                    if line.contains("uint32")
                        && let Some(id_str) = line.split_whitespace().last()
                        && let Ok(id) = id_str.parse::<u32>()
                    {
                        return Ok(id);
                    }
                }
                // Compute a deterministic hash based on notification content
                let id = simple_hash(&format!("{}{}", notification.title, notification.body));
                Ok(id)
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("DBus notification failed: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to send DBus notification: {}", e)),
    }
}

/// Simple hash for notification content (consistent across calls)
fn simple_hash(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    hash & 0x7FFFFFFF
}

/// Close a notification by ID
#[expect(dead_code, reason = "notification dismissal API")]
pub fn close_notification(id: u32) -> Result<(), String> {
    let output = Command::new("dbus-send")
        .args([
            "--session",
            "--dest=org.freedesktop.Notifications",
            "--type=method_call",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications.CloseNotification",
        ])
        .arg(format!("uint32:{}", id))
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to close notification: {}", error))
            }
        }
        Err(e) => Err(format!("DBus error: {}", e)),
    }
}

/// Send a notification using the best available backend (DBus > notify-send)
///
/// Attempts to use DBus for richer notification features (icons, urgency, timeouts).
/// Falls back to notify-send if DBus is unavailable.
/// This provides graceful degradation while maintaining maximum compatibility.
pub fn send_notification_smart(notification: &Notification) -> Result<(), String> {
    let app_name = "OpenCrust";

    // Try DBus first for richer features
    if let Ok(_id) = send_notification_dbus(app_name, notification) {
        return Ok(());
    }

    // Fall back to notify-send if DBus fails
    send_notification(notification)
}

/// Send a simple notification (convenience function)
#[expect(dead_code, reason = "convenience notification wrapper")]
pub fn notify(title: impl Into<String>, body: impl Into<String>) -> Result<(), String> {
    let notification = Notification::new(title, body);
    send_notification(&notification)
}

/// Send a notification with options
#[expect(dead_code, reason = "convenience notification wrapper")]
pub fn notify_with_options(
    title: impl Into<String>,
    body: impl Into<String>,
    options: NotificationOptions,
) -> Result<(), String> {
    let notification = Notification {
        title: title.into(),
        body: body.into(),
        options,
    };
    send_notification(&notification)
}

/// Show a transient notification that auto-dismiss after duration
#[expect(dead_code, reason = "convenience notification wrapper")]
pub fn notify_timed(
    title: impl Into<String>,
    body: impl Into<String>,
    duration: Duration,
) -> Result<(), String> {
    let seconds = duration.as_secs() as u32;
    let notification = Notification::new(title, body).with_expire_timeout(seconds);
    send_notification(&notification)
}

/// Show an error notification
#[expect(dead_code, reason = "convenience notification wrapper")]
pub fn notify_error(title: impl Into<String>, body: impl Into<String>) -> Result<(), String> {
    let notification = Notification::new(title, body).with_urgency(NotificationUrgency::Critical);
    send_notification(&notification)
}

/// Show a success notification
#[expect(dead_code, reason = "convenience notification wrapper")]
pub fn notify_success(title: impl Into<String>, body: impl Into<String>) -> Result<(), String> {
    let notification = Notification::new(title, body)
        .with_urgency(NotificationUrgency::Low)
        .with_icon("dialog-information");
    send_notification(&notification)
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
