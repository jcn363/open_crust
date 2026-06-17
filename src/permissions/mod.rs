//! Security permission system for tool execution
//!
//! Controls which tools and inputs are allowed, denied, or require user
//! confirmation. Supports glob-pattern matching on inputs, per-tool rules,
//! and global defaults. Used by the LLM loop to gate all tool calls.

use crate::config::{Config, PermissionAction, PermissionRule};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Role-based access control for OpenCrust operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Role {
    /// Full access to all operations
    Admin,
    /// Can read/write files, run tools, but no system-level changes
    #[default]
    Developer,
    /// Read-only access, can review but not modify
    Reviewer,
}

/// Permission template defining allowed operations per role.
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code, reason = "public API for role-based access control")]
pub struct RoleTemplate {
    pub role: Role,
    pub can_write_files: bool,
    pub can_execute_commands: bool,
    pub can_manage_mcp: bool,
    pub can_manage_plugins: bool,
    pub can_modify_config: bool,
    pub blocked_path_prefixes: Vec<String>,
}

impl RoleTemplate {
    /// Create a default template for the given role.
    #[allow(dead_code, reason = "public API for role-based access control")]
    pub fn for_role(role: Role) -> Self {
        match role {
            Role::Admin => RoleTemplate {
                role,
                can_write_files: true,
                can_execute_commands: true,
                can_manage_mcp: true,
                can_manage_plugins: true,
                can_modify_config: true,
                blocked_path_prefixes: vec!["/proc".to_string(), "/sys".to_string()],
            },
            Role::Developer => RoleTemplate {
                role,
                can_write_files: true,
                can_execute_commands: true,
                can_manage_mcp: false,
                can_manage_plugins: false,
                can_modify_config: false,
                blocked_path_prefixes: vec![
                    "/root".to_string(),
                    "/etc/shadow".to_string(),
                    "/etc/passwd".to_string(),
                ],
            },
            Role::Reviewer => RoleTemplate {
                role,
                can_write_files: false,
                can_execute_commands: false,
                can_manage_mcp: false,
                can_manage_plugins: false,
                can_modify_config: false,
                blocked_path_prefixes: vec![],
            },
        }
    }

    /// Check if an operation is allowed by this role template.
    #[allow(dead_code, reason = "public API for role-based access control")]
    pub fn check_operation(&self, operation: &str) -> Result<(), String> {
        match operation {
            "write_file" if !self.can_write_files => {
                Err(format!("Role {:?} cannot write files", self.role))
            }
            "execute_command" if !self.can_execute_commands => {
                Err(format!("Role {:?} cannot execute commands", self.role))
            }
            "manage_mcp" if !self.can_manage_mcp => {
                Err(format!("Role {:?} cannot manage MCP servers", self.role))
            }
            "manage_plugins" if !self.can_manage_plugins => {
                Err(format!("Role {:?} cannot manage plugins", self.role))
            }
            "modify_config" if !self.can_modify_config => {
                Err(format!("Role {:?} cannot modify configuration", self.role))
            }
            _ => Ok(()),
        }
    }

    /// Check if a file path is blocked by this role template.
    #[allow(dead_code, reason = "public API for role-based access control")]
    pub fn check_path(&self, path: &str) -> Result<(), String> {
        for blocked in &self.blocked_path_prefixes {
            if path.starts_with(blocked) {
                return Err(format!(
                    "Path '{}' is blocked for role {:?}",
                    path, self.role
                ));
            }
        }
        Ok(())
    }
}

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
mod tests;
