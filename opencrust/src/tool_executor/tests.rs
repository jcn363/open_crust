use crate::config::Config;
use crate::custom_tools::CustomToolManager;
use crate::lsp::LspManager;
use crate::mcp::McpManager;
use crate::orchestrator::Orchestrator;
use crate::permissions::PermissionManager;
use crate::planner::Planner;
use crate::plugins::PluginManager;
use crate::rag::RagManager;
use crate::skills::SkillManager;
use crate::tool_executor::ToolExecutor;
use crate::web::WebManager;
use lru::LruCache;
use serde_json::json;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

fn make_executor() -> ToolExecutor {
    let mcp_manager = Arc::new(Mutex::new(McpManager::new()));
    let lsp_manager = Arc::new(Mutex::new(LspManager::new()));
    let skill_manager = Arc::new(Mutex::new(SkillManager::new()));
    let custom_tool_manager = Arc::new(Mutex::new(CustomToolManager::new()));
    let config = Arc::new(Config::default());
    let permission_manager = Arc::new(PermissionManager::new(config.clone()));
    let web_manager = Arc::new(WebManager::new().expect("Failed to create web manager"));
    let planner = Arc::new(Mutex::new(Planner::new()));
    let rag_manager = Arc::new(Mutex::new(RagManager::new(&config)));
    let pinned_files = Arc::new(Mutex::new(Vec::new()));
    let orchestrator = Arc::new(Mutex::new(Orchestrator::new(config.clone())));
    let plugin_manager = Arc::new(Mutex::new(PluginManager::new()));

    ToolExecutor::new(
        config.clone(),
        mcp_manager,
        lsp_manager,
        skill_manager,
        custom_tool_manager,
        permission_manager,
        web_manager,
        planner,
        rag_manager,
        pinned_files,
        orchestrator,
        plugin_manager,
    )
}

#[tokio::test]
async fn test_pin_unpin() {
    let tool_executor = make_executor();

    let pin_args = json!({ "path": "/test/file.rs" });
    let result = tool_executor.execute_pin(&pin_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Successfully pinned: /test/file.rs");

    let result = tool_executor.execute_pin(&pin_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Already pinned: /test/file.rs");

    let unpin_args = json!({ "path": "/test/file.rs" });
    let result = tool_executor.execute_unpin(&unpin_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Successfully unpinned: /test/file.rs");

    let result = tool_executor.execute_unpin(&unpin_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Not pinned: /test/file.rs");

    let empty_args = json!({ "path": "" });
    let result = tool_executor.execute_pin(&empty_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Error: No path provided for pinning");

    let result = tool_executor.execute_unpin(&empty_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Error: No path provided for unpinning");
}

#[tokio::test]
async fn file_cache_returns_cached_content_within_ttl() {
    let tool_executor = make_executor();

    let mut cache = tool_executor.file_cache.lock().await;
    cache.put(
        "test_cache_file.rs".to_string(),
        ("cached content here".to_string(), Instant::now()),
    );

    let entry = cache.get("test_cache_file.rs");
    assert!(entry.is_some());
    let (content, timestamp) = entry.unwrap();
    assert_eq!(content, "cached content here");
    assert!(timestamp.elapsed() < Duration::from_secs(3600));
}

#[tokio::test]
async fn file_cache_evicts_expired_entries() {
    let tool_executor = make_executor();

    let mut cache = tool_executor.file_cache.lock().await;
    let expired_time = Instant::now() - Duration::from_secs(7200);
    cache.put(
        "expired_file.rs".to_string(),
        ("old content".to_string(), expired_time),
    );

    let entry = cache.get("expired_file.rs");
    assert!(entry.is_some());
    let (_, timestamp) = entry.unwrap();
    assert!(timestamp.elapsed() >= Duration::from_secs(3600));
}

#[tokio::test]
async fn file_cache_lru_eviction_at_capacity() {
    let mut cache: LruCache<String, (String, Instant)> =
        LruCache::new(NonZeroUsize::new(3).unwrap());

    cache.put(
        "a.rs".to_string(),
        ("content_a".to_string(), Instant::now()),
    );
    cache.put(
        "b.rs".to_string(),
        ("content_b".to_string(), Instant::now()),
    );
    cache.put(
        "c.rs".to_string(),
        ("content_c".to_string(), Instant::now()),
    );

    assert_eq!(cache.len(), 3);

    cache.put(
        "d.rs".to_string(),
        ("content_d".to_string(), Instant::now()),
    );

    assert_eq!(cache.len(), 3);
    assert!(cache.get("a.rs").is_none());
    assert!(cache.get("b.rs").is_some());
    assert!(cache.get("c.rs").is_some());
    assert!(cache.get("d.rs").is_some());
}

#[tokio::test]
async fn file_cache_lru_access_refreshes_position() {
    let mut cache: LruCache<String, (String, Instant)> =
        LruCache::new(NonZeroUsize::new(3).unwrap());

    cache.put(
        "a.rs".to_string(),
        ("content_a".to_string(), Instant::now()),
    );
    cache.put(
        "b.rs".to_string(),
        ("content_b".to_string(), Instant::now()),
    );
    cache.put(
        "c.rs".to_string(),
        ("content_c".to_string(), Instant::now()),
    );

    cache.get("a.rs");

    cache.put(
        "d.rs".to_string(),
        ("content_d".to_string(), Instant::now()),
    );

    assert!(cache.get("a.rs").is_some());
    assert!(cache.get("b.rs").is_none());
    assert!(cache.get("c.rs").is_some());
    assert!(cache.get("d.rs").is_some());
}
