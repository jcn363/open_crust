//! File context injection for prompts
//!
//! Reads file contents into the current prompt using `@file` references.
//! Supports glob patterns and validates paths through the security layer.

use crate::security::validate_path;
use std::fs;

pub fn inject_file_context(prompt: &str) -> String {
    let mut enriched_prompt = String::from(prompt);
    let mut files_to_inject = Vec::new();

    for word in prompt.split_whitespace() {
        if word.starts_with('@') && word.len() > 1 {
            let path = &word[1..];
            let path = path.trim_end_matches(&[',', '.', ';', ':', '?', '!'][..]);

            // Validate the path before adding
            match validate_path(path) {
                Ok(valid_path) => {
                    let path_str = valid_path.to_string_lossy().to_string();
                    if fs::metadata(&valid_path).is_ok() && !files_to_inject.contains(&path_str) {
                        files_to_inject.push(path_str);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Skipping invalid path '@{}': {}", path, e);
                }
            }
        }
    }

    if !files_to_inject.is_empty() {
        enriched_prompt.push_str("\n\n---\nFile Contexts:\n");
        for path in files_to_inject {
            if let Ok(content) = fs::read_to_string(&path) {
                enriched_prompt.push_str(&format!(
                    "\n<file path=\"{}\">\n{}\n</file>\n",
                    path, content
                ));
            }
        }
    }

    enriched_prompt
}
