//! Auto memory system — persists conversation learnings across sessions
//!
//! Extracts patterns from conversations (preferences, decisions, conventions)
//! and stores them for recall in future sessions. Inspired by Claude Code's
//! auto-memory feature.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Memory categories for classifying stored entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    UserPreference,
    ProjectContext,
    LearnedPattern,
    CodeConvention,
    Decision,
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryCategory::UserPreference => write!(f, "Preference"),
            MemoryCategory::ProjectContext => write!(f, "Project"),
            MemoryCategory::LearnedPattern => write!(f, "Pattern"),
            MemoryCategory::CodeConvention => write!(f, "Convention"),
            MemoryCategory::Decision => write!(f, "Decision"),
        }
    }
}

/// A single memory entry persisted across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub category: MemoryCategory,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
}

/// Auto memory manager — stores and retrieves learned patterns.
pub struct AutoMemory {
    entries: HashMap<String, MemoryEntry>,
    memory_dir: PathBuf,
}

impl AutoMemory {
    /// Create a new auto-memory instance backed by the given directory.
    pub fn new(memory_dir: PathBuf) -> Self {
        let mut manager = Self {
            entries: HashMap::new(),
            memory_dir,
        };
        let _ = manager.load();
        manager
    }

    /// Store a memory entry. Overwrites existing entry with same key.
    pub fn remember(&mut self, key: &str, value: &str, category: MemoryCategory) {
        let now = Utc::now();
        let entry = MemoryEntry {
            key: key.to_string(),
            value: value.to_string(),
            category,
            created_at: now,
            last_accessed: now,
            access_count: 0,
        };
        self.entries.insert(key.to_string(), entry);
        let _ = self.save();
    }

    /// Recall memories matching a fuzzy query on key or value.
    pub fn recall(&mut self, query: &str) -> Vec<MemoryEntry> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<MemoryEntry> = self
            .entries
            .values()
            .filter(|e| {
                e.key.to_lowercase().contains(&query_lower)
                    || e.value.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        // Update access metadata
        let now = Utc::now();
        for entry in &mut results {
            if let Some(stored) = self.entries.get_mut(&entry.key) {
                stored.last_accessed = now;
                stored.access_count += 1;
            }
        }

        results.sort_by_key(|b| std::cmp::Reverse(b.access_count));
        results
    }

    /// Remove a memory by key.
    pub fn forget(&mut self, key: &str) -> bool {
        let removed = self.entries.remove(key).is_some();
        if removed {
            let _ = self.save();
        }
        removed
    }

    /// List all memories in a given category.
    pub fn list_by_category(&self, category: &MemoryCategory) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| &e.category == category)
            .collect()
    }

    /// List all memories.
    pub fn list_all(&self) -> Vec<&MemoryEntry> {
        self.entries.values().collect()
    }

    /// Auto-extract memories from conversation messages.
    /// Scans for patterns like "I prefer...", "Let's use...", etc.
    pub fn auto_extract(&mut self, messages: &[String]) {
        for message in messages {
            let lower = message.to_lowercase();

            // User preferences
            if lower.contains("i prefer") || lower.contains("i like") || lower.contains("i want") {
                let key = extract_pattern_key(message, "preference");
                self.remember(&key, message, MemoryCategory::UserPreference);
            }

            // Decisions
            if lower.contains("let's use")
                || lower.contains("we should")
                || lower.contains("i'll use")
                || lower.contains("going with")
            {
                let key = extract_pattern_key(message, "decision");
                self.remember(&key, message, MemoryCategory::Decision);
            }

            // Code conventions
            if lower.contains("always use")
                || lower.contains("never use")
                || lower.contains("the pattern is")
                || lower.contains("convention is")
            {
                let key = extract_pattern_key(message, "convention");
                self.remember(&key, message, MemoryCategory::CodeConvention);
            }
        }
    }

    /// Persist memories to disk as JSON.
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.memory_dir)?;
        let path = self.memory_dir.join("memory.json");
        let content = serde_json::to_string_pretty(&self.entries).map_err(std::io::Error::other)?;
        std::fs::write(path, content)
    }

    /// Load memories from disk.
    fn load(&mut self) -> std::io::Result<()> {
        let path = self.memory_dir.join("memory.json");
        if !path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&path)?;
        self.entries = serde_json::from_str(&content).map_err(std::io::Error::other)?;
        Ok(())
    }
}

/// Extract a short key from a message for storage.
fn extract_pattern_key(message: &str, prefix: &str) -> String {
    let words: Vec<&str> = message.split_whitespace().take(5).collect();
    let slug = words.join("-").to_lowercase();
    // Remove non-alphanumeric characters except hyphens
    let clean: String = slug
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();
    format!("{}:{}", prefix, clean)
}

impl Default for AutoMemory {
    fn default() -> Self {
        let memory_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/opencrust/memory");
        Self::new(memory_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_memory_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("opencrust_memory_test");
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn remember_and_recall() {
        let dir = test_memory_dir();
        let mut memory = AutoMemory::new(dir.clone());
        memory.remember("test-key", "test-value", MemoryCategory::UserPreference);

        let results = memory.recall("test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "test-key");
        assert_eq!(results[0].value, "test-value");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn forget_removes_entry() {
        let dir = test_memory_dir();
        let mut memory = AutoMemory::new(dir.clone());
        memory.remember("to-forget", "value", MemoryCategory::Decision);
        assert!(memory.forget("to-forget"));
        assert!(memory.recall("to-forget").is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_by_category_filters() {
        let dir = test_memory_dir();
        let mut memory = AutoMemory::new(dir.clone());
        memory.remember("pref1", "value", MemoryCategory::UserPreference);
        memory.remember("conv1", "value", MemoryCategory::CodeConvention);

        let prefs = memory.list_by_category(&MemoryCategory::UserPreference);
        assert_eq!(prefs.len(), 1);
        assert_eq!(prefs[0].key, "pref1");

        let convs = memory.list_by_category(&MemoryCategory::CodeConvention);
        assert_eq!(convs.len(), 1);
        assert_eq!(convs[0].key, "conv1");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn auto_extract_detects_preferences() {
        let dir = test_memory_dir();
        let mut memory = AutoMemory::new(dir.clone());
        let messages = vec!["I prefer dark mode for the editor".to_string()];
        memory.auto_extract(&messages);
        assert!(!memory.recall("dark mode").is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn auto_extract_detects_decisions() {
        let dir = test_memory_dir();
        let mut memory = AutoMemory::new(dir.clone());
        let messages = vec!["Let's use tokio for async runtime".to_string()];
        memory.auto_extract(&messages);
        assert!(!memory.recall("tokio").is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_and_load_persists_data() {
        let dir = test_memory_dir();
        {
            let mut memory = AutoMemory::new(dir.clone());
            memory.remember("persistent", "data", MemoryCategory::ProjectContext);
        }
        // Reload from disk
        let mut memory2 = AutoMemory::new(dir.clone());
        let results = memory2.recall("persistent");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "data");
        let _ = fs::remove_dir_all(&dir);
    }
}
