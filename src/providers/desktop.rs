//! Desktop environment provider trait
//!
//! Abstracts desktop environment detection and theming across different DEs.

use crate::desktop::detection::{CinnamonInfo, DesktopEnvironment, DisplayServer};
use crate::providers::Provider;

/// Desktop provider trait for environment detection and theming
pub trait DesktopProvider: Provider {
    /// Detect the current desktop environment
    fn detect(&self) -> DesktopEnvironment;

    /// Get detailed Cinnamon info (if applicable)
    fn cinnamon_info(&self) -> CinnamonInfo;

    /// Get display server protocol
    fn display_server(&self) -> DisplayServer;

    /// Check if the desktop supports theming
    fn supports_theming(&self) -> bool;

    /// Get theme colors for the current desktop
    fn theme_colors(&self) -> Option<ThemeColors>;
}

/// Theme colors extracted from desktop environment
#[derive(Debug, Clone, Default)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub border: String,
}

/// Default desktop provider using the existing detection module
pub struct DefaultDesktopProvider;

impl Provider for DefaultDesktopProvider {
    fn id(&self) -> &str {
        "default"
    }

    fn name(&self) -> &str {
        "Default Desktop Detection"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn priority(&self) -> u8 {
        10 // Low priority, used as fallback
    }
}

impl DesktopProvider for DefaultDesktopProvider {
    fn detect(&self) -> DesktopEnvironment {
        crate::desktop::detection::detect_desktop()
    }

    fn cinnamon_info(&self) -> CinnamonInfo {
        crate::desktop::detection::detect_cinnamon()
    }

    fn display_server(&self) -> DisplayServer {
        crate::desktop::detection::detect_display_server()
    }

    fn supports_theming(&self) -> bool {
        self.detect() == DesktopEnvironment::Cinnamon
    }

    fn theme_colors(&self) -> Option<ThemeColors> {
        let info = self.cinnamon_info();
        if info.desktop == DesktopEnvironment::Cinnamon {
            Some(ThemeColors {
                background: info.theme.background,
                foreground: info.theme.foreground,
                accent: info.theme.accent,
                border: info.theme.border,
            })
        } else {
            None
        }
    }
}

/// Registry for desktop providers
pub type DesktopProviderRegistry = crate::providers::ProviderRegistry<dyn DesktopProvider>;

/// Create default desktop provider registry
pub fn default_desktop_registry() -> DesktopProviderRegistry {
    let mut registry = DesktopProviderRegistry::new();
    registry.register(Box::new(DefaultDesktopProvider));
    registry
}
