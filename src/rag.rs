//! RAG (Retrieval-Augmented Generation) — vector store and semantic search
//!
//! Manages document embeddings and cosine-similarity search over the project
//! codebase. Indexes files via configurable chunking, stores embeddings with
//! metadata, and retrieves relevant context for LLM prompts.

use crate::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A stored embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbedding {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// Vector store for semantic search
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VectorStore {
    pub embeddings: HashMap<String, StoredEmbedding>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            embeddings: HashMap::new(),
        }
    }

    /// Load vector store from disk
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join("vectors.json");
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(store) => store,
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to parse vector store ({}), starting fresh",
                            e
                        );
                        Self::new()
                    }
                },
                Err(_) => Self::new(),
            }
        } else {
            Self::new()
        }
    }

    /// Save vector store to disk
    pub fn save(&self, config_dir: &Path) {
        let path = config_dir.join("vectors.json");
        if let Ok(content) = serde_json::to_string_pretty(self) {
            if let Err(e) = fs::create_dir_all(config_dir) {
                eprintln!("Warning: Failed to create vector store dir: {}", e);
            }
            if let Err(e) = fs::write(&path, &content) {
                eprintln!("Warning: Failed to write vector store: {}", e);
            }
        }
    }

    /// Add an embedding to the store
    pub fn add(&mut self, embedding: StoredEmbedding) {
        self.embeddings.insert(embedding.id.clone(), embedding);
    }

    /// Search for similar embeddings using cosine similarity
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(&StoredEmbedding, f32)> {
        let mut results: Vec<(&StoredEmbedding, f32)> = self
            .embeddings
            .values()
            .map(|emb| {
                let similarity = cosine_similarity(query_embedding, &emb.embedding);
                (emb, similarity)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(top_k).collect()
    }

    /// Clear all embeddings
    pub fn clear(&mut self) {
        self.embeddings.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        let num_embeddings = self.embeddings.len();
        let dim = self
            .embeddings
            .values()
            .next()
            .map(|e| e.embedding.len())
            .unwrap_or(0);
        (num_embeddings, dim)
    }
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let norm = norm_a.sqrt() * norm_b.sqrt();
    if norm == 0.0 { 0.0 } else { dot_product / norm }
}

/// Generate embeddings using Ollama API
pub async fn generate_embedding(ollama_url: &str, text: &str) -> Result<Vec<f32>, String> {
    let client = Client::new();
    let url = format!("{}/api/embeddings", ollama_url.trim_end_matches('/'));

    let payload = serde_json::json!({
        "model": "nomic-embed-text",  // Default embedding model for Ollama
        "prompt": text
    });

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to call Ollama embeddings API: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama API returned error: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    let embedding = json
        .get("embedding")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid response format from Ollama".to_string())?;

    let vec: Vec<f32> = embedding
        .iter()
        .filter_map(|v| v.as_f64().map(|f| f as f32))
        .collect();

    if vec.is_empty() {
        Err("Empty embedding returned from Ollama".to_string())
    } else {
        Ok(vec)
    }
}

/// Chunk text into smaller pieces for embedding, preserving line numbers
pub fn chunk_text(text: &str, max_chars: usize) -> Vec<(String, usize, usize)> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut line_start = 1;
    let mut current_line = 1;

    for line in text.lines() {
        if current_chunk.len() + line.len() + 1 > max_chars && !current_chunk.is_empty() {
            chunks.push((current_chunk.clone(), line_start, current_line));
            current_chunk.clear();
            line_start = current_line;
        }
        current_chunk.push_str(line);
        current_chunk.push('\n');
        current_line += 1;
    }

    if !current_chunk.is_empty() {
        chunks.push((current_chunk, line_start, current_line));
    }

    chunks
}

/// RAG manager — document indexing and semantic search
///
/// Indexes project files into a vector store with configurable chunking.
/// Provides cosine-similarity search to retrieve relevant context for
/// LLM prompts. Persists embeddings across restarts.
pub struct RagManager {
    vector_store: VectorStore,
    config_dir: std::path::PathBuf,
    ollama_url: String,
}

impl RagManager {
    pub fn new(config: &Config) -> Self {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/opencrust");

        let ollama_url = config
            .ollama_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());

        let vector_store = VectorStore::load(&config_dir);

        Self {
            vector_store,
            config_dir,
            ollama_url,
        }
    }

    /// Index a file by generating embeddings for its chunks
    /// Note: Does not save the vector store automatically. Call save() or flush() when done indexing.
    pub async fn index_file(&mut self, file_path: &str) -> Result<usize, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

        let chunks = chunk_text(&content, 512);
        let mut indexed = 0;

        for (i, (chunk, line_start, line_end)) in chunks.iter().enumerate() {
            match generate_embedding(&self.ollama_url, chunk).await {
                Ok(embedding) => {
                    let stored = StoredEmbedding {
                        id: format!("{}:{}", file_path, i),
                        content: chunk.clone(),
                        embedding,
                        file_path: file_path.to_string(),
                        line_start: *line_start,
                        line_end: *line_end,
                    };
                    self.vector_store.add(stored);
                    indexed += 1;
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to embed chunk {} of {}: {}",
                        i, file_path, e
                    );
                }
            }
        }

        Ok(indexed)
    }

    /// Recursively index all code files in a directory
    /// Returns (files_indexed, total_chunks)
    pub async fn index_codebase(&mut self, root: &str) -> Result<(usize, usize), String> {
        let root_path = PathBuf::from(root);
        if !root_path.exists() {
            return Err(format!("Root path does not exist: {}", root));
        }

        let mut files_indexed = 0;
        let mut total_chunks = 0;

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let file_path = entry.path();
            let file_path_str = file_path.to_string_lossy().to_string();

            if is_code_file(&file_path_str) {
                match self.index_file(&file_path_str).await {
                    Ok(chunks) => {
                        files_indexed += 1;
                        total_chunks += chunks;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to index file {}: {}", file_path_str, e);
                    }
                }
            }
        }

        // Save the vector store after batch indexing
        self.flush()?;

        Ok((files_indexed, total_chunks))
    }

    /// Save the vector store to disk
    /// Should be called after batch indexing operations
    pub fn flush(&self) -> Result<(), String> {
        self.vector_store.save(&self.config_dir);
        Ok(())
    }

    /// Perform semantic search
    pub async fn semantic_search(&self, query: &str, top_k: usize) -> String {
        // Generate embedding for the query
        let query_embedding = match generate_embedding(&self.ollama_url, query).await {
            Ok(emb) => emb,
            Err(e) => return format!("Error generating embedding: {}", e),
        };

        // Search for similar embeddings
        let results = self.vector_store.search(&query_embedding, top_k);

        if results.is_empty() {
            format!("No semantic matches found for '{}'.", query)
        } else {
            let mut output = format!("Semantic search results for '{}':\n", query);
            for (emb, similarity) in results {
                output.push_str(&format!(
                    "\n[{}] {} (lines {}-{}, similarity: {:.3})\n{}\n",
                    emb.file_path,
                    emb.file_path,
                    emb.line_start,
                    emb.line_end,
                    similarity,
                    emb.content.lines().take(5).collect::<Vec<_>>().join("\n")
                ));
            }
            output
        }
    }

    /// Clear all indexed data
    #[expect(dead_code, reason = "public API for CodeIndexer")]
    pub fn clear_index(&mut self) {
        self.vector_store.clear();
        self.vector_store.save(&self.config_dir);
    }

    /// Get index statistics
    #[expect(dead_code, reason = "public API for CodeIndexer")]
    pub fn stats(&self) -> (usize, usize) {
        self.vector_store.stats()
    }
}

/// Check if a file is a code file based on extension
fn is_code_file(path: &str) -> bool {
    let code_extensions = [
        "rs", "py", "js", "ts", "jsx", "tsx", "go", "java", "c", "cpp", "h", "hpp", "rb", "php",
        "swift", "kt", "cs", "sh", "bash", "zsh", "toml", "yaml", "yml", "json", "md",
    ];

    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| code_extensions.contains(&ext))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "Identical vectors should have similarity 1.0"
        );
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!(
            sim.abs() < 1e-6,
            "Orthogonal vectors should have similarity 0.0"
        );
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![-1.0, 0.0, 0.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!(
            (sim - (-1.0)).abs() < 1e-6,
            "Opposite vectors should have similarity -1.0"
        );
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let v1 = vec![];
        let v2 = vec![];
        let sim = cosine_similarity(&v1, &v2);
        assert_eq!(sim, 0.0, "Empty vectors should return 0.0");
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let v1 = vec![1.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&v1, &v2);
        assert_eq!(sim, 0.0, "Different length vectors should return 0.0");
    }

    #[test]
    fn test_chunk_text() {
        let text = "line1\nline2\nline3\nline4\nline5";
        let chunks = chunk_text(text, 20);
        assert!(!chunks.is_empty(), "Should produce at least one chunk");
        for chunk in &chunks {
            assert!(chunk.0.len() <= 20, "Chunk should not exceed max_chars");
        }
    }

    #[test]
    fn test_chunk_text_small() {
        let text = "short";
        let chunks = chunk_text(text, 100);
        assert_eq!(chunks.len(), 1, "Short text should produce one chunk");
    }

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file("test.rs"));
        assert!(is_code_file("test.py"));
        assert!(is_code_file("test.js"));
        assert!(!is_code_file("test.txt"));
        assert!(!is_code_file("test.pdf"));
    }

    #[test]
    fn test_vector_store_new() {
        let store = VectorStore::new();
        assert_eq!(store.embeddings.len(), 0);
    }

    #[test]
    fn test_vector_store_add_and_search() {
        let mut store = VectorStore::new();

        let embedding = StoredEmbedding {
            id: "test1".to_string(),
            content: "test content".to_string(),
            embedding: vec![1.0, 0.0, 0.0],
            file_path: "test.rs".to_string(),
            line_start: 1,
            line_end: 3,
        };

        store.add(embedding);
        assert_eq!(store.embeddings.len(), 1);

        let query = vec![1.0, 0.0, 0.0];
        let results = store.search(&query, 5);
        assert!(!results.is_empty(), "Should find similar embedding");
        assert!(
            (results[0].1 - 1.0).abs() < 1e-6,
            "Should find exact match with similarity 1.0"
        );
    }

    #[tokio::test]
    #[ignore = "Requires Ollama running with nomic-embed-text model locally"]
    async fn test_generate_embedding() -> Result<(), Box<dyn std::error::Error>> {
        // This test requires Ollama running with nomic-embed-text model
        if std::process::Command::new("ollama")
            .args(["list"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("nomic-embed-text"))
            .unwrap_or(false)
        {
            let result = generate_embedding("http://localhost:11434", "test").await;
            assert!(result.is_ok(), "Should generate embedding");
            let embedding = result?;
            assert!(!embedding.is_empty(), "Embedding should not be empty");
            assert_eq!(
                embedding.len(),
                768,
                "nomic-embed-text should produce 768-dim embeddings"
            );
        }
        Ok(())
    }
}
