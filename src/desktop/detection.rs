//! Linux Mint Cinnamon desktop environment detection
//!
//! Detects if running on Linux Mint Cinnamon and extracts desktop-specific
//! configuration like theme colors, icon paths, and system settings.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Desktop environment type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    /// Linux Mint Cinnamon desktop
    Cinnamon,
    /// MATE desktop (Mint's predecessor)
    Mate,
    /// KDE Plasma desktop
    Plasma,
    /// GNOME desktop
    Gnome,
    /// Xfce desktop
    Xfce,
    /// Unknown desktop environment
    Unknown,
}

/// Display server protocol
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DisplayServer {
    /// X11 display server
    X11,
    /// Wayland display server
    Wayland,
    /// Unknown display server
    #[default]
    Unknown,
}

impl DisplayServer {
    /// Check if running on Wayland
    #[allow(dead_code)]
    pub fn is_wayland(&self) -> bool {
        matches!(self, DisplayServer::Wayland)
    }

    /// Check if running on X11
    #[allow(dead_code)]
    pub fn is_x11(&self) -> bool {
        matches!(self, DisplayServer::X11)
    }

    /// Get display server name
    pub fn name(&self) -> &str {
        match self {
            DisplayServer::X11 => "X11",
            DisplayServer::Wayland => "Wayland",
            DisplayServer::Unknown => "Unknown",
        }
    }
}

impl DesktopEnvironment {
    /// Check if running on Linux Mint Cinnamon
    pub fn is_cinnamon(&self) -> bool {
        matches!(self, DesktopEnvironment::Cinnamon)
    }

    /// Get desktop environment name as string
    pub fn name(&self) -> &str {
        match self {
            DesktopEnvironment::Cinnamon => "Cinnamon",
            DesktopEnvironment::Mate => "MATE",
            DesktopEnvironment::Plasma => "KDE Plasma",
            DesktopEnvironment::Gnome => "GNOME",
            DesktopEnvironment::Xfce => "Xfce",
            DesktopEnvironment::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for DesktopEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Cinnamon-specific theme colors extracted from desktop
#[derive(Debug, Clone, Default)]
pub struct CinnamonTheme {
    /// Background color (hex format)
    pub background: String,
    /// Foreground/text color (hex format)
    pub foreground: String,
    /// Accent/primary color (hex format)
    pub accent: String,
    /// Secondary color (hex format)
    #[allow(dead_code)]
    pub secondary: String,
    /// Border color (hex format)
    pub border: String,
    /// Error/warning color (hex format)
    #[allow(dead_code)]
    pub error: String,
}

impl CinnamonTheme {
    /// Create default theme (matches default Mint dark theme)
    pub fn default_dark() -> Self {
        Self {
            background: "#2d2d2d".to_string(),
            foreground: "#e0e0e0".to_string(),
            accent: "#3eb8ac".to_string(),
            secondary: "#5c5c5c".to_string(),
            border: "#404040".to_string(),
            error: "#e74c3c".to_string(),
        }
    }

    /// Create default theme (matches default Mint light theme)
    pub fn default_light() -> Self {
        Self {
            background: "#f7f7f7".to_string(),
            foreground: "#2d2d2d".to_string(),
            accent: "#3eb8ac".to_string(),
            secondary: "#e0e0e0".to_string(),
            border: "#d0d0d0".to_string(),
            error: "#e74c3c".to_string(),
        }
    }
}

/// Cinnamon system information
#[derive(Debug, Clone)]
pub struct CinnamonInfo {
    /// Desktop environment variant
    pub desktop: DesktopEnvironment,
    /// Display server protocol (X11/Wayland)
    pub display_server: DisplayServer,
    /// Mint version (if available)
    pub version: Option<String>,
    /// User's theme settings
    pub theme: CinnamonTheme,
    /// Home directory path
    #[allow(dead_code)]
    pub home_dir: PathBuf,
    /// Config directory path
    #[allow(dead_code)]
    pub config_dir: PathBuf,
    /// Data directory path
    #[allow(dead_code)]
    pub data_dir: PathBuf,
    /// Icon theme
    pub icon_theme: String,
    /// Cursor theme
    pub cursor_theme: String,
}

impl Default for CinnamonInfo {
    fn default() -> Self {
        Self {
            desktop: DesktopEnvironment::Unknown,
            display_server: DisplayServer::Unknown,
            version: None,
            theme: CinnamonTheme::default_dark(),
            home_dir: dirs::home_dir().unwrap_or_default(),
            config_dir: dirs::config_dir().unwrap_or_default(),
            data_dir: dirs::data_dir().unwrap_or_default(),
            icon_theme: "Mint-Y".to_string(),
            cursor_theme: "DMZ".to_string(),
        }
    }
}

/// Detect the current desktop environment
pub fn detect_desktop() -> DesktopEnvironment {
    // Check XDG_CURRENT_DESKTOP
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop_lower = desktop.to_lowercase();
        if desktop_lower.contains("cinnamon") {
            return DesktopEnvironment::Cinnamon;
        } else if desktop_lower.contains("mate") {
            return DesktopEnvironment::Mate;
        } else if desktop_lower.contains("kde") {
            return DesktopEnvironment::Plasma;
        } else if desktop_lower.contains("gnome") {
            return DesktopEnvironment::Gnome;
        } else if desktop_lower.contains("xfce") {
            return DesktopEnvironment::Xfce;
        }
    }

    // Check DESKTOP_SESSION (fallback)
    if let Ok(session) = env::var("DESKTOP_SESSION") {
        let session_lower = session.to_lowercase();
        if session_lower.contains("cinnamon") {
            return DesktopEnvironment::Cinnamon;
        } else if session_lower.contains("mate") {
            return DesktopEnvironment::Mate;
        } else if session_lower.contains("kde") {
            return DesktopEnvironment::Plasma;
        } else if session_lower.contains("gnome") {
            return DesktopEnvironment::Gnome;
        } else if session_lower.contains("xfce") {
            return DesktopEnvironment::Xfce;
        }
    }

    // Check for cinnamon-specific processes
    if let Ok(proc_cmdline) = fs::read_to_string("/proc/1/cmdline")
        && proc_cmdline.to_lowercase().contains("cinnamon")
    {
        return DesktopEnvironment::Cinnamon;
    }

    DesktopEnvironment::Unknown
}

/// Detect the display server protocol (X11 or Wayland)
pub fn detect_display_server() -> DisplayServer {
    // Check WAYLAND_DISPLAY (most reliable Wayland indicator)
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return DisplayServer::Wayland;
    }

    // Check XDG_SESSION_TYPE
    if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => return DisplayServer::Wayland,
            "x11" => return DisplayServer::X11,
            _ => {}
        }
    }

    // Check for X11-specific variables
    if env::var("DISPLAY").is_ok() {
        return DisplayServer::X11;
    }

    DisplayServer::Unknown
}

/// Read a gsettings value from Cinnamon schema
pub fn gsettings_get(schema: &str, key: &str) -> Option<String> {
    // Try gsettings CLI first (more reliable on Mint)
    let output = std::process::Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()?;

    if output.status.success() {
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Remove quotes if present
        let value = value.trim().trim_matches('"').to_string();
        if !value.is_empty() && value != "none" {
            return Some(value);
        }
    }

    None
}

/// Read Cinnamon theme from gsettings
pub fn detect_cinnamon_theme() -> CinnamonTheme {
    // Check if theme is dark or light mode
    let is_dark = gsettings_get("org.cinnamon.desklets", "use-dark-theme")
        .or_else(|| gsettings_get("org.cinnamon.theme", "name"))
        .map(|name| name.to_lowercase().contains("dark"))
        .unwrap_or(true); // Default to dark theme

    // Try to get specific colors from cinnamon theme settings
    let accent = gsettings_get("org.cinnamon.desklets", "primary-color")
        .unwrap_or_else(|| "#3eb8ac".to_string());

    let theme = if is_dark {
        CinnamonTheme::default_dark()
    } else {
        CinnamonTheme::default_light()
    };

    CinnamonTheme { accent, ..theme }
}

/// Detect icon and cursor themes
pub fn detect_cinnamon_themes() -> (String, String) {
    let icon_theme = gsettings_get("org.gnome.desktop.interface", "icon-theme")
        .unwrap_or_else(|| "Mint-Y".to_string());

    let cursor_theme = gsettings_get("org.gnome.desktop.interface", "cursor-theme")
        .unwrap_or_else(|| "DMZ".to_string());

    (icon_theme, cursor_theme)
}

/// Get Linux Mint version
pub fn get_mint_version() -> Option<String> {
    // Check /etc/linuxmint/info
    if let Ok(contents) = fs::read_to_string("/etc/linuxmint/info") {
        for line in contents.lines() {
            if line.starts_with("VERSION=") {
                return Some(line.trim_start_matches("VERSION=").to_string());
            }
        }
    }

    // Try lsb_release as fallback
    let output = std::process::Command::new("lsb_release")
        .args(["-rs"])
        .output()
        .ok()?;

    if output.status.success() {
        return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    None
}

/// Detect Cinnamon desktop environment with full info
pub fn detect_cinnamon() -> CinnamonInfo {
    let desktop = detect_desktop();
    let display_server = detect_display_server();
    let is_cinnamon = desktop == DesktopEnvironment::Cinnamon;

    let theme = if is_cinnamon {
        detect_cinnamon_theme()
    } else {
        CinnamonTheme::default_dark()
    };

    let (icon_theme, cursor_theme) = if is_cinnamon {
        detect_cinnamon_themes()
    } else {
        ("Mint-Y".to_string(), "DMZ".to_string())
    };

    let version = if is_cinnamon {
        get_mint_version()
    } else {
        None
    };

    CinnamonInfo {
        desktop,
        display_server,
        version,
        theme,
        home_dir: dirs::home_dir().unwrap_or_default(),
        config_dir: dirs::config_dir().unwrap_or_default(),
        data_dir: dirs::data_dir().unwrap_or_default(),
        icon_theme,
        cursor_theme,
    }
}

/// Check if running on a supported desktop environment (Cinnamon, Mate, Gnome, Plasma)
pub fn is_supported_desktop() -> bool {
    let desktop = detect_desktop();
    matches!(
        desktop,
        DesktopEnvironment::Cinnamon
            | DesktopEnvironment::Mate
            | DesktopEnvironment::Gnome
            | DesktopEnvironment::Plasma
    )
}

/// Get the Cinnamon info, or return defaults if not detected
pub fn get_cinnamon_info() -> CinnamonInfo {
    detect_cinnamon()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_detection() {
        let desktop = detect_desktop();
        // Just check it's a valid variant
        assert!(matches!(
            desktop,
            DesktopEnvironment::Cinnamon | DesktopEnvironment::Unknown
        ));
    }

    #[test]
    fn test_theme_defaults() {
        let dark = CinnamonTheme::default_dark();
        assert_eq!(dark.background, "#2d2d2d");
        assert_eq!(dark.foreground, "#e0e0e0");

        let light = CinnamonTheme::default_light();
        assert_eq!(light.background, "#f7f7f7");
    }
}
