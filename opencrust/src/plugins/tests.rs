use crate::plugins::PluginManager;
use std::fs;
use std::path::Path;
use uuid::Uuid;

fn create_test_manifest(dir: &Path, name: &str, enabled: bool) -> std::path::PathBuf {
    fs::create_dir_all(dir).unwrap();
    let manifest = dir.join("plugin.json");
    let content = serde_json::json!({
        "name": name,
        "version": "1.0.0",
        "description": "test plugin",
        "author": "test",
        "enabled": enabled
    });
    fs::write(&manifest, serde_json::to_string_pretty(&content).unwrap()).unwrap();
    manifest
}

#[test]
fn test_discover_plugins() {
    let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
    let plugin_dir = dir.join("test-plugin");
    create_test_manifest(&plugin_dir, "test-plugin", true);

    let mut mgr = PluginManager::new();
    mgr.search_paths = vec![dir.clone()];
    let discovered = mgr.discover();
    assert!(discovered.contains(&"test-plugin".to_string()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_list_plugins() {
    let mut mgr = PluginManager::new();
    let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
    let plugin_dir = dir.join("plugin-a");
    create_test_manifest(&plugin_dir, "plugin-a", true);
    mgr.search_paths = vec![dir.clone()];
    mgr.discover();

    let list = mgr.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "plugin-a");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_enable_disable() {
    let mut mgr = PluginManager::new();
    let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
    let plugin_dir = dir.join("toggle-plugin");
    create_test_manifest(&plugin_dir, "toggle-plugin", true);
    mgr.search_paths = vec![dir.clone()];
    mgr.discover();

    assert!(mgr.get("toggle-plugin").unwrap().enabled);
    mgr.disable("toggle-plugin").unwrap();
    assert!(!mgr.get("toggle-plugin").unwrap().enabled);
    mgr.enable("toggle-plugin").unwrap();
    assert!(mgr.get("toggle-plugin").unwrap().enabled);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_stats() {
    let mut mgr = PluginManager::new();
    let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
    let d1 = dir.join("p1");
    let d2 = dir.join("p2");
    create_test_manifest(&d1, "p1", true);
    create_test_manifest(&d2, "p2", false);
    mgr.search_paths = vec![dir.clone()];
    mgr.discover();

    let stats = mgr.stats();
    assert_eq!(stats.total, 2);
    assert_eq!(stats.enabled, 1);
    assert_eq!(stats.disabled, 1);
    assert_eq!(stats.hook_count, 0);
    assert_eq!(stats.tool_count, 0);
    assert_eq!(stats.citation_count, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_get_nonexistent() {
    let mgr = PluginManager::new();
    assert!(mgr.get("nope").is_none());
}

#[test]
fn test_enable_nonexistent() {
    let mut mgr = PluginManager::new();
    assert!(mgr.enable("nope").is_err());
}

#[test]
fn test_invalid_manifest() {
    let dir = std::env::temp_dir().join(format!("opencrust_plugins_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("plugin.json"), "not json").unwrap();

    let mut mgr = PluginManager::new();
    mgr.search_paths = vec![dir.clone()];
    let discovered = mgr.discover();
    assert!(discovered.is_empty());

    let _ = fs::remove_dir_all(&dir);
}
