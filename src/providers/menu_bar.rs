//! Menu bar provider trait
//!
//! Abstracts menu bar integration across different platforms.
//! Currently only macOS is supported via the `macos-menu-bar` feature.

use crate::providers::Provider;

/// Menu bar status states
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MenuBarStatus {
    /// Idle state
    #[default]
    Idle,
    /// Working state
    Working,
    /// Error state
    Error,
    /// Custom status with text
    Custom(String),
}

/// Menu bar provider trait for native menu bar integration
pub trait MenuBarProvider: Provider {
    /// Start the menu bar (should be called in a background thread)
    fn start(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Stop the menu bar
    fn stop(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Update the status icon
    fn set_status(&self, status: MenuBarStatus) -> Result<(), Box<dyn std::error::Error>>;

    /// Update the agent count displayed in menu
    fn set_agent_count(&self, count: usize) -> Result<(), Box<dyn std::error::Error>>;

    /// Show a notification badge with text
    fn show_badge(&self, text: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Hide the notification badge
    fn hide_badge(&self) -> Result<(), Box<dyn std::error::Error>>;
}

/// macOS menu bar provider (only available with macos-menu-bar feature)
#[cfg(all(target_os = "macos", feature = "macos-menu-bar"))]
pub struct MacOSMenuBarProvider;

#[cfg(all(target_os = "macos", feature = "macos-menu-bar"))]
impl Provider for MacOSMenuBarProvider {
    fn id(&self) -> &str {
        "macos-menu-bar"
    }

    fn name(&self) -> &str {
        "macOS Menu Bar"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn priority(&self) -> u8 {
        100 // Highest priority on macOS
    }
}

#[cfg(all(target_os = "macos", feature = "macos-menu-bar"))]
impl MenuBarProvider for MacOSMenuBarProvider {
    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start menu bar in background thread
        let _ = crate::desktop::menu_bar::start_menu_bar();
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Menu bar runs until app quit
        Ok(())
    }

    fn set_status(&self, _status: MenuBarStatus) -> Result<(), Box<dyn std::error::Error>> {
        // Status updates are handled via channels
        Ok(())
    }

    fn set_agent_count(&self, _count: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Agent count updates are handled via channels
        Ok(())
    }

    fn show_badge(&self, _text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Badge updates are handled via channels
        Ok(())
    }

    fn hide_badge(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Badge updates are handled via channels
        Ok(())
    }
}

/// Fallback menu bar provider for unsupported platforms
pub struct FallbackMenuBarProvider;

impl Provider for FallbackMenuBarProvider {
    fn id(&self) -> &str {
        "fallback-menu-bar"
    }

    fn name(&self) -> &str {
        "Fallback Menu Bar"
    }

    fn is_available(&self) -> bool {
        true // Always available as fallback
    }

    fn priority(&self) -> u8 {
        0 // Lowest priority
    }
}

impl MenuBarProvider for FallbackMenuBarProvider {
    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(()) // No-op
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(()) // No-op
    }

    fn set_status(&self, _status: MenuBarStatus) -> Result<(), Box<dyn std::error::Error>> {
        Ok(()) // No-op
    }

    fn set_agent_count(&self, _count: usize) -> Result<(), Box<dyn std::error::Error>> {
        Ok(()) // No-op
    }

    fn show_badge(&self, _text: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(()) // No-op
    }

    fn hide_badge(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(()) // No-op
    }
}

/// Registry for menu bar providers
pub type MenuBarProviderRegistry = crate::providers::ProviderRegistry<dyn MenuBarProvider>;

/// Create default menu bar provider registry
pub fn default_menu_bar_registry() -> MenuBarProviderRegistry {
    let mut registry = MenuBarProviderRegistry::new();

    #[cfg(all(target_os = "macos", feature = "macos-menu-bar"))]
    registry.register(Box::new(MacOSMenuBarProvider));

    // Always register fallback
    registry.register(Box::new(FallbackMenuBarProvider));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_bar_status_default() {
        let status = MenuBarStatus::default();
        assert_eq!(status, MenuBarStatus::Idle);
    }

    #[test]
    fn test_fallback_provider() {
        let provider = FallbackMenuBarProvider;
        assert_eq!(provider.id(), "fallback-menu-bar");
        // Use Provider trait's is_available method
        assert!(provider.is_available()); // Provider trait returns true
    }

    #[test]
    fn test_registry() {
        let registry = default_menu_bar_registry();
        let available = registry.available();
        // At least fallback should be registered
        assert!(!available.is_empty());
    }
}
