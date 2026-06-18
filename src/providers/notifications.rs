//! Notifications provider trait
//!
//! Abstracts system notification delivery across different platforms and backends.

use crate::desktop::notifications::mod_types::{Notification, NotificationDaemon, NotificationUrgency};
use crate::providers::Provider;
use std::time::Duration;

/// Notification provider trait for sending system notifications
pub trait NotificationProvider: Provider {
    /// Check if this notification backend is available
    fn check_daemon(&self) -> NotificationDaemon;

    /// Send a notification
    fn send(&self, notification: &Notification) -> Result<u32, String>;

    /// Send a timed notification (auto-dismiss)
    fn notify_timed(
        &self,
        title: String,
        body: String,
        duration: Duration,
    ) -> Result<(), String> {
        let seconds = duration.as_secs() as u32;
        let notification = Notification::new(title, body).with_expire_timeout(seconds);
        self.send(&notification).map(|_| ())
    }

    /// Send an error notification
    fn notify_error(&self, title: String, body: String) -> Result<(), String> {
        let notification = Notification::new(title, body).with_urgency(NotificationUrgency::Critical);
        self.send(&notification).map(|_| ())
    }

    /// Send a success notification
    fn notify_success(&self, title: String, body: String) -> Result<(), String> {
        let notification = Notification::new(title, body)
            .with_urgency(NotificationUrgency::Low)
            .with_icon("dialog-information");
        self.send(&notification).map(|_| ())
    }
}

/// Linux notification provider using notify-send/DBus
pub struct LinuxNotificationProvider;

impl Provider for LinuxNotificationProvider {
    fn id(&self) -> &str {
        "linux-notify"
    }

    fn name(&self) -> &str {
        "Linux notify-send / DBus"
    }

    fn is_available(&self) -> bool {
        crate::desktop::notifications::linux::check_notification_daemon_linux().available
    }

    fn priority(&self) -> u8 {
        80
    }
}

impl NotificationProvider for LinuxNotificationProvider {
    fn check_daemon(&self) -> NotificationDaemon {
        crate::desktop::notifications::linux::check_notification_daemon_linux()
    }

    fn send(&self, notification: &Notification) -> Result<u32, String> {
        let app_name = "OpenCrust";
        // Try DBus first
        if let Ok(id) = crate::desktop::notifications::linux::send_notification_dbus(app_name, notification) {
            return Ok(id);
        }
        // Fall back to notify-send
        crate::desktop::notifications::linux::send_notification(notification).map(|_| 0)
    }
}

/// macOS notification provider using osascript
#[cfg(target_os = "macos")]
pub struct MacOSNotificationProvider;

#[cfg(target_os = "macos")]
impl Provider for MacOSNotificationProvider {
    fn id(&self) -> &str {
        "macos-notify"
    }

    fn name(&self) -> &str {
        "macOS osascript"
    }

    fn is_available(&self) -> bool {
        crate::desktop::notifications::macos::is_macos_notifications_available()
    }

    fn priority(&self) -> u8 {
        90
    }
}

#[cfg(target_os = "macos")]
impl NotificationProvider for MacOSNotificationProvider {
    fn check_daemon(&self) -> NotificationDaemon {
        crate::desktop::notifications::macos::check_notification_daemon_macos()
    }

    fn send(&self, notification: &Notification) -> Result<u32, String> {
        crate::desktop::notifications::macos::send_notification_macos(notification)
    }
}

/// Registry for notification providers
pub type NotificationProviderRegistry = crate::providers::ProviderRegistry<dyn NotificationProvider>;

/// Create default notification provider registry
pub fn default_notification_registry() -> NotificationProviderRegistry {
    let mut registry = NotificationProviderRegistry::new();
    registry.register(Box::new(LinuxNotificationProvider));
    #[cfg(target_os = "macos")]
    registry.register(Box::new(MacOSNotificationProvider));
    registry
}