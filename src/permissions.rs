//! Security permission system for tool execution
//!
//! Controls which tools and inputs are allowed, denied, or require user
//! confirmation. Supports glob-pattern matching on inputs, per-tool rules,
//! and global defaults. Used by the LLM loop to gate all tool calls.

use crate::config::{Config, PermissionAction, PermissionRule};
use glob::Pattern;
use std::sync::Arc;

/// Security gate for all tool execution
///
/// Checks tool names and inputs against configurable rules. Returns
/// Allow/Deny/Ask decisions based on glob-pattern matching and
/// per-tool or global permission rules.
pub struct PermissionManager {
    config: Arc<Config>,
}

impl PermissionManager {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub fn check_permission(&self, tool_name: &str, input: &str) -> PermissionAction {
        // 1. Check tool-specific rules
        if let Some(rule) = self.config.permission.get(tool_name) {
            match rule {
                PermissionRule::Action(action) => return action.clone(),
                PermissionRule::Map(map) => {
                    // Collect matching patterns first, then take last key in insertion order
                    let mut keys: Vec<&String> = map
                        .keys()
                        .filter(|k| {
                            if *k == "*" {
                                return true;
                            }
                            Pattern::new(k).map(|p| p.matches(input)).unwrap_or(false)
                        })
                        .collect();
                    // Sort for deterministic evaluation: non-glob patterns first, then globs.
                    // Among same category, preserve stable order for predictable matching.
                    keys.sort_by(|a, b| {
                        let a_star = a.contains('*');
                        let b_star = b.contains('*');
                        a_star.cmp(&b_star)
                    });
                    // Last key after sort is the most specific match (glob > non-glob,
                    // and among same category the last insertion-order match wins)
                    if let Some(last_key) = keys.last() {
                        return map.get(*last_key).cloned().unwrap_or(PermissionAction::Ask);
                    }
                    return PermissionAction::Ask;
                }
            }
        }

        // 2. Check global rule
        if let Some(rule) = self.config.permission.get("*") {
            match rule {
                PermissionRule::Action(action) => return action.clone(),
                PermissionRule::Map(_) => {} // Global * shouldn't be a map really
            }
        }

        PermissionAction::Ask
    }

    /// Verification helper: Check if a tool is allowed without prompting
    pub fn is_allowed_without_prompt(&self, tool_name: &str, input: &str) -> bool {
        matches!(
            self.check_permission(tool_name, input),
            PermissionAction::Allow
        )
    }

    pub fn check_network_permission(&self, url_str: &str) -> bool {
        if self.config.allowed_domains.is_empty() {
            return true;
        }

        if let Ok(parsed_url) = url::Url::parse(url_str)
            && let Some(host) = parsed_url.host_str()
        {
            return self.config.allowed_domains.iter().any(|domain| {
                // Exact match OR subdomain match (prefix with dot to prevent suffix bypass)
                host == domain || host.ends_with(&format!(".{}", domain))
            });
        }
        false
    }
}

#[cfg(test)]
mod tests {
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
}
