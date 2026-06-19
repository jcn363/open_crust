//! Shared notification types used by platform backends

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

    /// Parse from urgency name (e.g., "low", "normal", "critical")
    pub fn from_name(s: &str) -> Self {
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
}

impl NotificationCategory {
    /// Get DBus category string
    pub fn to_dbus(&self) -> &str {
        "general"
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
}

/// Notification daemon status
#[derive(Debug, Clone, Default)]
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
