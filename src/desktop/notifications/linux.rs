//! Linux notification backend — notify-send and DBus

use std::process::Command;

use super::mod_types::{Notification, NotificationDaemon, NotificationUrgency};

/// Send a notification using notify-send (most compatible method)
pub fn send_notification(notification: &Notification) -> Result<(), String> {
    let mut args = vec![
        "-u".to_string(),
        notification.options.urgency.as_arg().to_string(),
    ];

    // Add expire timeout if specified
    if notification.options.expire_timeout > 0 {
        args.push("-t".to_string());
        args.push((notification.options.expire_timeout * 1000).to_string());
    }

    // Add icon if specified
    if let Some(ref icon) = notification.options.icon {
        args.push("-i".to_string());
        args.push(icon.clone());
    }

    // Add category if specified
    if notification.options.category != super::mod_types::NotificationCategory::General {
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
pub fn send_notification_dbus(app_name: &str, notification: &Notification) -> Result<u32, String> {
    let urgency: u32 = match notification.options.urgency {
        NotificationUrgency::Low => 0,
        NotificationUrgency::Normal => 1,
        NotificationUrgency::Critical => 2,
    };

    let expire_timeout = if notification.options.expire_timeout > 0 {
        notification.options.expire_timeout as i32 * 1000
    } else {
        -1
    };

    let icon = notification
        .options
        .icon
        .clone()
        .unwrap_or_else(|| "dialog-information".to_string());

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
        .arg("uint32:0")
        .arg(format!("string:{}", icon))
        .arg(format!("string:{}", notification.title))
        .arg(format!("string:{}", notification.body))
        .arg({
            if let Some(ref action) = notification.options.action {
                format!("array:string:default,{}", action)
            } else {
                "array:string:[]".to_string()
            }
        })
        .arg(format!("dict:string:variant:urgency,byte:{}", urgency))
        .arg(format!("int32:{}", expire_timeout))
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                for line in response.lines() {
                    if line.contains("uint32")
                        && let Some(id_str) = line.split_whitespace().last()
                        && let Ok(id) = id_str.parse::<u32>()
                    {
                        return Ok(id);
                    }
                }
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
pub fn simple_hash(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    hash & 0x7FFFFFFF
}

/// Close a notification by ID
#[expect(
    dead_code,
    reason = "notification management API for programmatic dismissal"
)]
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

/// Check notification daemon availability on Linux via DBus/notify-send.
pub fn check_notification_daemon_linux() -> NotificationDaemon {
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
