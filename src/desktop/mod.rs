//! Cinnamon desktop integration module
//!
//! Provides desktop environment detection, system notifications, and native file pickers
//! for Linux Mint Cinnamon while keeping the core TUI unchanged.

pub mod detection;
pub mod file_picker;
pub mod notifications;

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
