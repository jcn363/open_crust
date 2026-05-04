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
                        if let Ok(pattern) = Pattern::new(pattern_str) {
                            if pattern.matches(input) {
                                result = action.clone();
                            }
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
}
