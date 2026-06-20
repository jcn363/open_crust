//! Cognitive memory types for agent persistence
//!
//! Inspired by the Engram memory ecosystem, these types enable
//! typed memory with lifecycle management, hybrid retrieval,
//! and knowledge graph integration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Cognitive memory types with different storage/retrieval semantics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    /// Events tied to specific moments ("debugged schema migration at 3pm")
    Episodic,
    /// Durable facts ("prefers TypeScript over JavaScript")
    Semantic,
    /// Behavioral rules ("always use Result<T> for error handling")
    Procedural,
    /// Active context (current task, open files, recent errors)
    Working,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Episodic => write!(f, "episodic"),
            Self::Semantic => write!(f, "semantic"),
            Self::Procedural => write!(f, "procedural"),
            Self::Working => write!(f, "working"),
        }
    }
}

/// Memory tier for lifecycle management (promotion/demotion)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTier {
    /// Ephemeral, session-only (expires quickly)
    Scratch,
    /// Daily consolidation window
    Daily,
    /// Short-term retention (14 days)
    ShortTerm,
    /// Long-term storage (90 days)
    LongTerm,
    /// Archived, rarely accessed
    Archive,
}

impl MemoryTier {
    /// Default TTL in days for each tier
    pub fn ttl_days(&self) -> u32 {
        match self {
            Self::Scratch => 1,
            Self::Daily => 2,
            Self::ShortTerm => 14,
            Self::LongTerm => 90,
            Self::Archive => 365,
        }
    }
}

/// Where a memory originated from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryOrigin {
    /// Explicitly stored by the user
    User,
    /// Auto-extracted from conversation
    Extracted,
    /// Imported from external source
    Imported,
    /// Produced by consolidation/sleep cycle
    Derived,
}

/// A single memory unit with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub content: String,
    pub memory_type: MemoryType,
    pub importance: f32,
    pub confidence: f32,
    pub tier: MemoryTier,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub tags: Vec<String>,
    pub entities: Vec<String>,
    pub origin: MemoryOrigin,
    pub ttl_days: Option<u32>,
    pub source_file: Option<String>,
}

impl Memory {
    /// Create a new memory with sensible defaults
    pub fn new(content: String, memory_type: MemoryType, origin: MemoryOrigin) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content,
            memory_type,
            importance: 0.5,
            confidence: 1.0,
            tier: MemoryTier::Scratch,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
            entities: Vec::new(),
            origin,
            ttl_days: None,
            source_file: None,
        }
    }

    /// Check if this memory has expired based on its tier
    pub fn is_expired(&self) -> bool {
        let ttl = self.ttl_days.unwrap_or_else(|| self.tier.ttl_days());
        let elapsed = Utc::now() - self.last_accessed;
        elapsed.num_days() > i64::from(ttl)
    }

    /// Record an access, updating counters
    pub fn record_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
        // Boost confidence slightly on access
        self.confidence = (self.confidence + 0.05).min(1.0);
    }
}

/// Predicate type for knowledge graph edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateType {
    /// "User uses TypeScript"
    Uses,
    /// "User prefers Rust"
    Prefers,
    /// "Module A depends on Module B"
    DependsOn,
    /// "Bug X caused by Change Y"
    CausedBy,
    /// "Decision A supersedes Decision B"
    Supersedes,
    /// "Fact A contradicts Fact B"
    Contradicts,
    /// "Memory A elaborates on Memory B"
    Elaborates,
    /// "Function is part of Module"
    PartOf,
    /// "Project is instance of Template"
    InstanceOf,
    /// "Code calls function"
    Calls,
    /// "File imports module"
    Imports,
    /// "Generic relationship"
    RelatedTo,
}

impl std::fmt::Display for PredicateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uses => write!(f, "uses"),
            Self::Prefers => write!(f, "prefers"),
            Self::DependsOn => write!(f, "depends_on"),
            Self::CausedBy => write!(f, "caused_by"),
            Self::Supersedes => write!(f, "supersedes"),
            Self::Contradicts => write!(f, "contradicts"),
            Self::Elaborates => write!(f, "elaborates"),
            Self::PartOf => write!(f, "part_of"),
            Self::InstanceOf => write!(f, "instance_of"),
            Self::Calls => write!(f, "calls"),
            Self::Imports => write!(f, "imports"),
            Self::RelatedTo => write!(f, "related_to"),
        }
    }
}

/// A knowledge graph triple (subject → predicate → object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: PredicateType,
    pub object: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub source_memory_ids: Vec<Uuid>,
}

impl KnowledgeTriple {
    /// Check if this triple is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let after_start = self.valid_from.map_or(true, |v| v <= now);
        let before_end = self.valid_to.map_or(true, |v| v > now);
        after_start && before_end
    }
}

/// Retrieval signal for hybrid search scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalSignal {
    pub vector_score: f32,
    pub keyword_score: f32,
    pub recency_score: f32,
    pub confidence_score: f32,
    pub feedback_score: f32,
    pub graph_boost: f32,
    pub temporal_boost: f32,
}

impl RetrievalSignal {
    /// Combined score using weighted fusion
    pub fn combined(&self) -> f32 {
        self.vector_score * 0.45
            + self.keyword_score * 0.15
            + self.recency_score * 0.15
            + self.confidence_score * 0.15
            + self.feedback_score * 0.10
            + self.graph_boost
            + self.temporal_boost
    }
}

/// User feedback on a memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Feedback {
    /// Memory was helpful
    Helpful,
    /// Memory was incorrect, user corrected it
    Corrected,
    /// Memory was irrelevant to the query
    Irrelevant,
}

/// Session handoff for context transfer between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHandoff {
    pub timestamp: DateTime<Utc>,
    pub current_task: String,
    pub completed: Vec<String>,
    pub next_steps: Vec<String>,
    pub open_questions: Vec<String>,
    pub file_references: Vec<String>,
    pub decisions: Vec<String>,
    pub notes: String,
}

impl Default for SessionHandoff {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            current_task: String::new(),
            completed: Vec::new(),
            next_steps: Vec::new(),
            open_questions: Vec::new(),
            file_references: Vec::new(),
            decisions: Vec::new(),
            notes: String::new(),
        }
    }
}

/// Memory categories for classifying stored entries (legacy auto-memory system).
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

/// A single memory entry persisted across sessions (legacy auto-memory system).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub category: MemoryCategory,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
}

/// Auto memory manager — stores and retrieves learned patterns (legacy auto-memory system).
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
