use super::*;
use crate::config::{Config, PermissionAction, PermissionRule};
use std::sync::Arc;

fn cfg_with_perms(perms: std::collections::HashMap<String, PermissionRule>) -> Arc<Config> {
    Arc::new(Config {
        permission: perms,
        allowed_domains: vec![],
        ..Default::default()
    })
}

// --- check_permission ---

#[test]
fn check_permission_exact_tool_action() {
    let mut perms = std::collections::HashMap::new();
    perms.insert(
        "bash".to_string(),
        PermissionRule::Action(PermissionAction::Deny),
    );
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    assert_eq!(
        mgr.check_permission("bash", "anything"),
        PermissionAction::Deny
    );
}

#[test]
fn check_permission_unknown_tool_falls_to_ask() {
    let mut perms = std::collections::HashMap::new();
    perms.insert(
        "read".to_string(),
        PermissionRule::Action(PermissionAction::Allow),
    );
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    assert_eq!(
        mgr.check_permission("write", "anything"),
        PermissionAction::Ask
    );
}

#[test]
fn check_permission_global_wildcard_rule() {
    let mut perms = std::collections::HashMap::new();
    perms.insert(
        "*".to_string(),
        PermissionRule::Action(PermissionAction::Deny),
    );
    perms.insert(
        "read".to_string(),
        PermissionRule::Action(PermissionAction::Allow),
    );
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    // Tool-specific takes precedence
    assert_eq!(mgr.check_permission("read", "x"), PermissionAction::Allow);
    // Global fallback
    assert_eq!(mgr.check_permission("write", "x"), PermissionAction::Deny);
}

#[test]
fn check_permission_glob_wins_over_exact_match() {
    let mut map = std::collections::HashMap::new();
    map.insert("src/main.rs".to_string(), PermissionAction::Allow);
    map.insert("*.rs".to_string(), PermissionAction::Ask);
    let mut perms = std::collections::HashMap::new();
    perms.insert("read".to_string(), PermissionRule::Map(map));
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    // Glob patterns sort after non-glob, so last() picks the glob rule
    assert_eq!(
        mgr.check_permission("read", "src/main.rs"),
        PermissionAction::Ask
    );
}

#[test]
fn check_permission_glob_pattern_fallback() {
    let mut map = std::collections::HashMap::new();
    map.insert("src/main.rs".to_string(), PermissionAction::Allow);
    map.insert("*.rs".to_string(), PermissionAction::Ask);
    let mut perms = std::collections::HashMap::new();
    perms.insert("read".to_string(), PermissionRule::Map(map));
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    // Glob match for .rs files (matches last in sorted order = most specific)
    // Non-glob patterns sort first, so *.rs comes last and wins
    assert_eq!(
        mgr.check_permission("read", "src/other.rs"),
        PermissionAction::Ask
    );
}

#[test]
fn check_permission_glob_wildcard_match_any() {
    let mut map = std::collections::HashMap::new();
    map.insert("*".to_string(), PermissionAction::Allow);
    let mut perms = std::collections::HashMap::new();
    perms.insert("bash".to_string(), PermissionRule::Map(map));
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    // * matches all inputs
    assert_eq!(
        mgr.check_permission("bash", "rm -rf /"),
        PermissionAction::Allow
    );
}

#[test]
fn check_permission_no_match_in_map_returns_ask() {
    let mut map = std::collections::HashMap::new();
    map.insert("/safe/path".to_string(), PermissionAction::Allow);
    let mut perms = std::collections::HashMap::new();
    perms.insert("read".to_string(), PermissionRule::Map(map));
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    // Input doesn't match any pattern
    assert_eq!(
        mgr.check_permission("read", "/unsafe/path"),
        PermissionAction::Ask
    );
}

// --- is_allowed_without_prompt ---

#[test]
fn is_allowed_returns_true_for_allow() {
    let mut perms = std::collections::HashMap::new();
    perms.insert(
        "read".to_string(),
        PermissionRule::Action(PermissionAction::Allow),
    );
    let mgr = PermissionManager::new(cfg_with_perms(perms));
    assert!(mgr.is_allowed_without_prompt("read", "anything"));
}

#[test]
fn is_allowed_returns_false_for_deny_or_ask() {
    let mgr = PermissionManager::new(Arc::new(Config::default()));
    assert!(!mgr.is_allowed_without_prompt("bash", "anything"));
}

// --- check_network_permission ---

#[test]
fn network_permit_empty_domain_list_allows_all() {
    let cfg = Arc::new(Config {
        allowed_domains: vec![],
        ..Default::default()
    });
    let mgr = PermissionManager::new(cfg);
    assert!(mgr.check_network_permission("https://evil.com"));
}

#[test]
fn network_permit_exact_match() {
    let cfg = Arc::new(Config {
        allowed_domains: vec!["github.com".to_string()],
        ..Default::default()
    });
    let mgr = PermissionManager::new(cfg);
    assert!(mgr.check_network_permission("https://github.com"));
}

#[test]
fn network_permit_subdomain_match() {
    let cfg = Arc::new(Config {
        allowed_domains: vec!["github.com".to_string()],
        ..Default::default()
    });
    let mgr = PermissionManager::new(cfg);
    assert!(mgr.check_network_permission("https://api.github.com"));
}

#[test]
fn network_deny_wrong_domain() {
    let cfg = Arc::new(Config {
        allowed_domains: vec!["github.com".to_string()],
        ..Default::default()
    });
    let mgr = PermissionManager::new(cfg);
    assert!(!mgr.check_network_permission("https://evil.com"));
}

#[test]
fn network_deny_suffix_bypass_attempt() {
    let cfg = Arc::new(Config {
        allowed_domains: vec!["github.com".to_string()],
        ..Default::default()
    });
    let mgr = PermissionManager::new(cfg);
    // evilgithub.com is NOT a subdomain of github.com
    assert!(!mgr.check_network_permission("https://evilgithub.com"));
}

#[test]
fn network_deny_invalid_url() {
    let cfg = Arc::new(Config {
        allowed_domains: vec!["github.com".to_string()],
        ..Default::default()
    });
    let mgr = PermissionManager::new(cfg);
    assert!(!mgr.check_network_permission("not-a-url"));
}
