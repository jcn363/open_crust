//! Desktop integration module
//!
//! Provides desktop environment detection, system notifications (Linux via notify-send/DBus,
//! macOS via osascript), native file pickers, and macOS menu bar integration.

pub mod detection;
pub mod file_picker;
pub mod notifications;

/// macOS menu bar integration (only available on macOS with macos-menu-bar feature)
#[cfg(all(target_os = "macos", feature = "macos-menu-bar"))]
pub mod menu_bar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Just verify the module compiles and exports the expected items
        let _ = detection::detect_desktop();
        // Test that the modules exist and can be used (without calling private test functions)
        let _ = file_picker::FilePickerBackend::KDialog;
        let _ = notifications::NotificationUrgency::Low;
    }
}
