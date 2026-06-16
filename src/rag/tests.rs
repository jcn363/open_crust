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
#[ignore = "Requires Ollama running with nomic-embed-text-v2-moe model locally"]
async fn test_generate_embedding() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires Ollama running with nomic-embed-text-v2-moe model
    if std::process::Command::new("ollama")
        .args(["list"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("nomic-embed-text-v2-moe"))
        .unwrap_or(false)
    {
        let result = generate_embedding("http://localhost:11434", "test").await;
        assert!(result.is_ok(), "Should generate embedding");
        let embedding = result?;
        assert!(!embedding.is_empty(), "Embedding should not be empty");
        // Note: nomic-embed-text-v2-moe embedding dimensions may vary
        // For now we just check it's not empty
        assert!(!embedding.is_empty(), "Embedding should not be empty");
    }
    Ok(())
}
