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
                    // Evaluate patterns, last one wins
                    let mut result = PermissionAction::Ask; // Default for map if no match
                    for (pattern_str, action) in map {
                        if pattern_str == "*" {
                            result = action.clone();
                            continue;
                        }
                        if let Ok(pattern) = Pattern::new(pattern_str)
                            && pattern.matches(input)
                        {
                            result = action.clone();
                        }
                    }
                    return result;
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
    #[allow(dead_code)]
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