//! File picker functionality for fuzzy file search

use crate::app::App;
use nucleo::{Config as NucleoConfig, Matcher, Utf32Str};
use rayon::prelude::*;
use walkdir::WalkDir;

impl App {
    /// Activate the file picker with an initial query string.
    /// Caches the full project file list and filters by the query.
    pub fn activate_file_picker(&mut self, query: String) {
        self.file_picker_active = true;
        self.file_picker_query = query.clone();
        self.file_picker_selected = 0;
        self.file_picker_scroll = 0;
        self.file_picker_last_input = Some(std::time::Instant::now());
        // Cache the full file list on first activation
        if self.cached_project_files.is_empty() {
            self.cached_project_files = self.collect_all_project_files();
            // Initialize nucleo matcher for fast fuzzy matching
            let config = NucleoConfig::DEFAULT;
            self.file_matcher = Some(Matcher::new(config));
        }
        self.file_picker_results = self.filter_project_files(&query);
    }

    /// Deactivate the file picker without selecting a file.
    pub fn cancel_file_picker(&mut self) {
        self.file_picker_active = false;
        self.file_picker_query.clear();
        self.file_picker_results.clear();
    }

    /// Confirm the current selection and insert the file path into input.
    pub fn confirm_file_picker(&mut self) -> Option<String> {
        if let Some(path) = self.file_picker_results.get(self.file_picker_selected) {
            let selected = path.clone();
            self.cancel_file_picker();
            Some(selected)
        } else {
            None
        }
    }

    /// Collect all files in the project (no filter). Used to populate cache.
    fn collect_all_project_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        for entry in WalkDir::new(".")
            .into_iter()
            .filter_entry(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| !n.starts_with('.'))
                    .unwrap_or(false)
            })
            .flatten()
        {
            if entry.file_type().is_file() {
                if let Some(path) = entry.path().to_str() {
                    // Skip binary/large files
                    if path.ends_with(".git")
                        || path.contains("/target/")
                        || path.contains("\\target\\")
                    {
                        continue;
                    }
                    files.push(path.to_string());
                }
            }
        }
        files.sort();
        files
    }

    /// Filter cached project files by query (fuzzy match). No filesystem access.
    /// Uses nucleo for fast fuzzy matching with parallel processing.
    pub(crate) fn filter_project_files(&mut self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return self
                .cached_project_files
                .iter()
                .take(100)
                .cloned()
                .collect();
        }

        // Debounce: only re-filter if 100ms has passed since last input
        if let Some(last_input) = self.file_picker_last_input {
            if last_input.elapsed() < std::time::Duration::from_millis(100) {
                // Return cached results if debounce period hasn't elapsed
                return self.file_picker_results.clone();
            }
        }
        self.file_picker_last_input = Some(std::time::Instant::now());

        let query_lower = query.to_lowercase();
        let _ = &mut self.file_matcher; // Ensure matcher is initialized

        // Use parallel processing for large file lists
        // Each thread gets its own buffers and matcher
        let scored: Vec<(u16, String)> = self
            .cached_project_files
            .par_iter()
            .filter_map(|f| {
                let mut haystack_buf = Vec::new();
                let mut needle_buf = Vec::new();
                let haystack = Utf32Str::new(f, &mut haystack_buf);
                let needle = Utf32Str::new(&query_lower, &mut needle_buf);
                // Create a local matcher for this thread
                let mut local_matcher = Matcher::new(NucleoConfig::DEFAULT);
                local_matcher
                    .fuzzy_match(haystack, needle)
                    .map(|score| (score, f.clone()))
            })
            .collect();

        // Sort by score (higher is better) and take top 100
        let mut scored = scored;
        scored.sort_by_key(|b| std::cmp::Reverse(b.0));
        scored.truncate(100);
        scored.into_iter().map(|(_, f)| f).collect()
    }
}
