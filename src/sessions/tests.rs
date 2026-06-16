use super::*;
use serde_json::json;

#[test]
fn test_fork_session() {
    let mgr = SessionManager::new();
    let test_id = "test-original";
    let messages = vec![json!({"role": "user", "content": "hello"})];

    // Save original
    mgr.save_session(test_id, &messages).unwrap();

    // Fork it
    let forked = mgr.fork_session(test_id, Some("test-fork")).unwrap();

    assert_eq!(forked.id, "test-fork");
    assert_eq!(forked.messages.len(), 1);
    assert_eq!(
        forked.messages[0],
        json!({"role": "user", "content": "hello"})
    );

    // Cleanup
    let _ = mgr.delete_session(test_id);
    let _ = mgr.delete_session("test-fork");
}

#[test]
fn test_fork_nonexistent() {
    let mgr = SessionManager::new();
    let result = mgr.fork_session("does-not-exist", None);
    assert!(result.is_err());
}

#[test]
fn test_fork_auto_name() {
    let mgr = SessionManager::new();
    let test_id = "test-original-2";
    let messages = vec![json!({"role": "user", "content": "test"})];

    // Save original
    mgr.save_session(test_id, &messages).unwrap();

    // Fork without providing name
    let forked = mgr.fork_session(test_id, None).unwrap();

    // Should contain original_id and "fork"
    assert!(forked.id.contains("test-original-2"));
    assert!(forked.id.contains("fork"));

    // Cleanup
    let _ = mgr.delete_session(test_id);
    let _ = mgr.delete_session(&forked.id);
}

#[test]
fn test_fork_duplicate_name() {
    let mgr = SessionManager::new();
    let test_id = "test-original-3";
    let messages = vec![json!({"role": "user", "content": "test"})];

    // Save original
    mgr.save_session(test_id, &messages).unwrap();

    // Create a session with the name we want to use
    mgr.save_session("test-fork-dup", &messages).unwrap();

    // Fork with a name that already exists
    let forked = mgr.fork_session(test_id, Some("test-fork-dup")).unwrap();

    // Should have a different name (with timestamp appended)
    assert_ne!(forked.id, "test-fork-dup");
    assert!(forked.id.contains("test-fork-dup"));

    // Cleanup
    let _ = mgr.delete_session(test_id);
    let _ = mgr.delete_session("test-fork-dup");
    let _ = mgr.delete_session(&forked.id);
}
