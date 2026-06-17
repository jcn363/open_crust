//! Cross-platform desktop environment detection
//!
//! Detects desktop environment on Linux (Cinnamon, MATE, Plasma, GNOME, Xfce),
//! macOS, and Windows with platform-specific details.

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
    /// macOS desktop (not yet detected, reserved for future cross-platform support)
    #[allow(dead_code, reason = "Reserved for future macOS support")]
    MacOS {
        /// macOS version (e.g., "14.5")
        version: String,
        /// Terminal emulator (e.g., "iTerm2", "Terminal.app", "Alacritty")
        terminal: String,
        /// User's shell (e.g., "/bin/zsh", "/bin/bash")
        shell: String,
    },
    /// Windows desktop (not yet detected, reserved for future cross-platform support)
    #[allow(dead_code, reason = "Reserved for future Windows support")]
    Windows {
        /// Windows version (e.g., "10.0.19045", "11.0.22631")
        version: String,
        /// Whether running in WSL
        is_wsl: bool,
        /// Terminal emulator (e.g., "Windows Terminal", "ConEmu", "cmd")
        terminal: String,
    },
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
    #[expect(dead_code, reason = "display server check API")]
    pub fn is_wayland(&self) -> bool {
        matches!(self, DisplayServer::Wayland)
    }

    /// Check if running on X11
    #[expect(dead_code, reason = "display server check API")]
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

    /// Check if running on macOS (reserved for future cross-platform support)
    #[allow(dead_code, reason = "Reserved for future macOS support")]
    pub fn is_macos(&self) -> bool {
        matches!(self, DesktopEnvironment::MacOS { .. })
    }

    /// Check if running on Windows (reserved for future cross-platform support)
    #[allow(dead_code, reason = "Reserved for future Windows support")]
    pub fn is_windows(&self) -> bool {
        matches!(self, DesktopEnvironment::Windows { .. })
    }

    /// Get desktop environment name as string
    pub fn name(&self) -> &str {
        match *self {
            DesktopEnvironment::Cinnamon => "Cinnamon",
            DesktopEnvironment::Mate => "MATE",
            DesktopEnvironment::Plasma => "KDE Plasma",
            DesktopEnvironment::Gnome => "GNOME",
            DesktopEnvironment::Xfce => "Xfce",
            DesktopEnvironment::MacOS { .. } => "macOS",
            DesktopEnvironment::Windows { .. } => "Windows",
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
    #[expect(dead_code, reason = "CinnamonTheme field")]
    pub secondary: String,
    /// Border color (hex format)
    pub border: String,
    /// Error/warning color (hex format)
    #[expect(dead_code, reason = "CinnamonTheme field")]
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
    #[expect(dead_code, reason = "Used in startup.rs for theme detection")]
    pub desktop: DesktopEnvironment,
    /// Display server protocol (X11/Wayland)
    pub display_server: DisplayServer,
    /// Mint version (if available)
    pub version: Option<String>,
    /// User's theme settings
    pub theme: CinnamonTheme,
    /// Home directory path
    #[expect(dead_code, reason = "CinnamonInfo field")]
    pub home_dir: PathBuf,
    /// Config directory path
    #[expect(dead_code, reason = "CinnamonInfo field")]
    pub config_dir: PathBuf,
    /// Data directory path
    #[expect(dead_code, reason = "CinnamonInfo field")]
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

    // Check for cinnamon-specific processes using pgrep (more reliable than /proc/1/cmdline)
    if std::process::Command::new("pgrep")
        .arg("-u")
        .arg(env::var("USER").unwrap_or_default())
        .arg("cinnamon")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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

/// Detect macOS desktop environment (reserved for future cross-platform support)
#[cfg(target_os = "macos")]
#[expect(dead_code, reason = "Reserved for future macOS support")]
pub fn detect_macos() -> DesktopEnvironment {
    // Get macOS version via sw_vers
    let version = std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Detect terminal emulator from TERM_PROGRAM
    let terminal = env::var("TERM_PROGRAM")
        .map(|s| match s.as_str() {
            "iTerm.app" => "iTerm2".to_string(),
            "Apple_Terminal" => "Terminal.app".to_string(),
            "vscode" => "VS Code".to_string(),
            _ => s,
        })
        .unwrap_or_else(|_| "unknown".to_string());

    // Detect shell from SHELL env var
    let shell = env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());

    DesktopEnvironment::MacOS {
        version,
        terminal,
        shell,
    }
}

/// Detect Windows desktop environment (reserved for future cross-platform support)
#[cfg(target_os = "windows")]
#[expect(dead_code, reason = "Reserved for future Windows support")]
pub fn detect_windows() -> DesktopEnvironment {
    // Get Windows version via ver command or systeminfo
    let version = std::process::Command::new("cmd")
        .args(["/c", "ver"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check if running in WSL
    let is_wsl = env::var("WSL_DISTRO_NAME").is_ok()
        || fs::read_to_string("/proc/version")
            .map(|s| s.to_lowercase().contains("microsoft"))
            .unwrap_or(false);

    // Detect terminal emulator
    let terminal = if env::var("WT_SESSION").is_ok() {
        "Windows Terminal".to_string()
    } else if env::var("CONEMU_PID").is_ok() {
        "ConEmu".to_string()
    } else if env::var("TERM").is_ok() {
        "Terminal".to_string()
    } else {
        "cmd.exe".to_string()
    };

    DesktopEnvironment::Windows {
        version,
        is_wsl,
        terminal,
    }
}

/// Cross-platform desktop detection entry point (reserved for future cross-platform support)
#[expect(dead_code, reason = "Reserved for future cross-platform support")]
pub fn detect_desktop_cross_platform() -> DesktopEnvironment {
    #[cfg(target_os = "macos")]
    {
        return detect_macos();
    }
    #[cfg(target_os = "windows")]
    {
        return detect_windows();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        detect_desktop()
    }
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
            DesktopEnvironment::Cinnamon
                | DesktopEnvironment::Mate
                | DesktopEnvironment::Plasma
                | DesktopEnvironment::Gnome
                | DesktopEnvironment::Xfce
                | DesktopEnvironment::MacOS { .. }
                | DesktopEnvironment::Windows { .. }
                | DesktopEnvironment::Unknown
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

    #[test]
    fn test_desktop_environment_name() {
        assert_eq!(DesktopEnvironment::Cinnamon.name(), "Cinnamon");
        assert_eq!(DesktopEnvironment::Mate.name(), "MATE");
        assert_eq!(DesktopEnvironment::Plasma.name(), "KDE Plasma");
        assert_eq!(DesktopEnvironment::Gnome.name(), "GNOME");
        assert_eq!(DesktopEnvironment::Xfce.name(), "Xfce");
        assert_eq!(DesktopEnvironment::Unknown.name(), "Unknown");

        let macos = DesktopEnvironment::MacOS {
            version: "14.5".to_string(),
            terminal: "iTerm2".to_string(),
            shell: "/bin/zsh".to_string(),
        };
        assert_eq!(macos.name(), "macOS");
        assert!(macos.is_macos());

        let windows = DesktopEnvironment::Windows {
            version: "10.0.19045".to_string(),
            is_wsl: false,
            terminal: "Windows Terminal".to_string(),
        };
        assert_eq!(windows.name(), "Windows");
        assert!(windows.is_windows());
    }
}
