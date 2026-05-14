use crate::config::{Config, PermissionAction, PermissionRule};
use glob::Pattern;
use std::sync::Arc;

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
