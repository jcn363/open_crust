use crate::config::{Config, PermissionAction, PermissionRule};
use glob::Pattern;

pub struct PermissionManager {
    config: Config,
}

impl PermissionManager {
    pub fn new(config: Config) -> Self {
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
                    // Sort for deterministic evaluation (glob patterns with * last)
                    keys.sort_by(|a, b| {
                        let a_star = a.contains('*');
                        let b_star = b.contains('*');
                        a_star.cmp(&b_star)
                    });
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
            return self
                .config
                .allowed_domains
                .iter()
                .any(|domain| host.ends_with(domain));
        }
        false
    }
}
