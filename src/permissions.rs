//! Security permission system for tool execution
//!
//! Controls which tools and inputs are allowed, denied, or require user
//! confirmation. Supports glob-pattern matching on inputs, per-tool rules,
//! and global defaults. Used by the LLM loop to gate all tool calls.

use crate::config::{Config, PermissionAction, PermissionRule};
use glob::Pattern;
use std::collections::HashMap;
use std::sync::Arc;

/// Pre-compiled permission pattern for efficient matching
struct CompiledPermissionRule {
    /// The original pattern string (for debugging)
    pattern: String,
    /// The compiled glob pattern
    compiled: Pattern,
    /// The action to take when this pattern matches
    action: PermissionAction,
}

/// Pre-compiled permission rules for a specific tool
struct CompiledToolRules {
    /// Simple action rule (Allow/Deny/Ask) - takes precedence over patterns
    simple_action: Option<PermissionAction>,
    /// Pattern-based rules, sorted by specificity (non-glob first, then glob)
    pattern_rules: Vec<CompiledPermissionRule>,
}

/// Security gate for all tool execution
///
/// Checks tool names and inputs against configurable rules. Returns
/// Allow/Deny/Ask decisions based on glob-pattern matching and
/// per-tool or global permission rules.
///
/// Patterns are pre-compiled at initialization for performance.
pub struct PermissionManager {
    config: Arc<Config>,
    /// Pre-compiled tool-specific rules
    tool_rules: HashMap<String, CompiledToolRules>,
    /// Pre-compiled global rules (for "*" tool)
    global_rules: Option<CompiledToolRules>,
}

impl PermissionManager {
    pub fn new(config: Arc<Config>) -> Self {
        let tool_rules = Self::compile_tool_rules(&config.permission);
        let global_rules = config.permission.get("*").map(Self::compile_rule);
        Self {
            config,
            tool_rules,
            global_rules,
        }
    }

    /// Compile a single permission rule into a CompiledToolRules structure
    fn compile_rule(rule: &PermissionRule) -> CompiledToolRules {
        match rule {
            PermissionRule::Action(action) => CompiledToolRules {
                simple_action: Some(action.clone()),
                pattern_rules: Vec::new(),
            },
            PermissionRule::Map(map) => {
                let mut pattern_rules: Vec<CompiledPermissionRule> = map
                    .iter()
                    .filter_map(|(pattern, action)| {
                        if pattern == "*" {
                            // Wildcard pattern matches everything
                            Some(CompiledPermissionRule {
                                pattern: pattern.clone(),
                                compiled: Pattern::new("*")
                                    .expect("Wildcard pattern should compile"),
                                action: action.clone(),
                            })
                        } else {
                            Pattern::new(pattern)
                                .ok()
                                .map(|compiled| CompiledPermissionRule {
                                    pattern: pattern.clone(),
                                    compiled,
                                    action: action.clone(),
                                })
                        }
                    })
                    .collect();

                // Sort for deterministic evaluation: non-glob patterns first, then globs.
                // Among same category, preserve stable order for predictable matching.
                pattern_rules.sort_by(|a, b| {
                    let a_star = a.pattern.contains('*');
                    let b_star = b.pattern.contains('*');
                    a_star.cmp(&b_star)
                });

                CompiledToolRules {
                    simple_action: None,
                    pattern_rules,
                }
            }
        }
    }

    /// Compile all tool-specific rules from the permission map
    fn compile_tool_rules(
        permissions: &HashMap<String, PermissionRule>,
    ) -> HashMap<String, CompiledToolRules> {
        permissions
            .iter()
            .filter(|(tool, _)| *tool != "*") // Skip global rule, handled separately
            .map(|(tool, rule)| (tool.clone(), Self::compile_rule(rule)))
            .collect()
    }

    pub fn check_permission(&self, tool_name: &str, input: &str) -> PermissionAction {
        // 1. Check tool-specific rules
        if let Some(rules) = self.tool_rules.get(tool_name) {
            // Simple action takes precedence
            if let Some(action) = &rules.simple_action {
                return action.clone();
            }
            // Check pattern rules - find all matches, return the last (most specific)
            let mut matched_action = None;
            for rule in &rules.pattern_rules {
                if rule.compiled.matches(input) {
                    matched_action = Some(rule.action.clone());
                }
            }
            if let Some(action) = matched_action {
                return action;
            }
            return PermissionAction::Ask;
        }

        // 2. Check global rule
        if let Some(rules) = &self.global_rules {
            if let Some(action) = &rules.simple_action {
                return action.clone();
            }
            let mut matched_action = None;
            for rule in &rules.pattern_rules {
                if rule.compiled.matches(input) {
                    matched_action = Some(rule.action.clone());
                }
            }
            if let Some(action) = matched_action {
                return action;
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
