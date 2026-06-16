//! Sidebar file tree and formatting

use crate::app::App;

impl App {
    /// Format the currently selected file in the sidebar.
    /// Returns a message describing the result.
    pub fn format_current_file(&mut self) -> String {
        if self.sidebar_items.is_empty() || self.sidebar_selected >= self.sidebar_items.len() {
            return "No file selected in sidebar".to_string();
        }
        let path = &self.sidebar_items[self.sidebar_selected];
        match crate::formatters::format_file(std::path::Path::new(path)) {
            Ok(_) => format!("Formatted {}", path),
            Err(e) => e,
        }
    }

    pub fn refresh_sidebar(&mut self) {
        if let Ok(entries) = std::fs::read_dir(".") {
            self.sidebar_items = entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            self.sidebar_items.sort();
        }
    }
}
