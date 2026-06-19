//! Compliance policy enforcement

use crate::audit::AuditEntry;
use serde::{Deserialize, Serialize};

use super::profiles::{PolicyRule, PolicySeverity, PolicyViolation};

/// A collection of policy rules for compliance enforcement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePolicy {
    pub name: String,
    pub description: String,
    pub rules: Vec<PolicyRule>,
}

impl CompliancePolicy {
    /// Create a default enterprise policy.
    pub fn enterprise_default() -> Self {
        Self {
            name: "Enterprise Default".into(),
            description: "Baseline enterprise compliance policy".into(),
            rules: vec![
                PolicyRule {
                    id: "POL-001".into(),
                    name: "Tool Approval Rate".into(),
                    description: "Overall tool approval rate must be >= 95%".into(),
                    severity: PolicySeverity::High,
                    tool_pattern: None,
                    max_denials: None,
                    min_approval_rate: Some(0.95),
                    enforced: true,
                },
                PolicyRule {
                    id: "POL-002".into(),
                    name: "Command Execution Control".into(),
                    description: "Shell command execution (bash) requires high approval".into(),
                    severity: PolicySeverity::Critical,
                    tool_pattern: Some("bash".into()),
                    max_denials: Some(5),
                    min_approval_rate: Some(0.90),
                    enforced: true,
                },
                PolicyRule {
                    id: "POL-003".into(),
                    name: "File Write Control".into(),
                    description: "File write operations must be audited".into(),
                    severity: PolicySeverity::High,
                    tool_pattern: Some("write".into()),
                    max_denials: Some(10),
                    min_approval_rate: None,
                    enforced: true,
                },
                PolicyRule {
                    id: "POL-004".into(),
                    name: "Network Access Audit".into(),
                    description: "All network/web operations logged".into(),
                    severity: PolicySeverity::Medium,
                    tool_pattern: Some("web_search".into()),
                    max_denials: None,
                    min_approval_rate: None,
                    enforced: false,
                },
            ],
        }
    }

    /// Evaluate all rules against audit entries.
    pub fn evaluate(&self, entries: &[AuditEntry]) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();
        let total = entries.len();
        let approved = entries.iter().filter(|e| e.approved).count();

        for rule in &self.rules {
            // Filter by tool pattern if specified
            let filtered: Vec<&AuditEntry> = match &rule.tool_pattern {
                Some(pattern) => entries
                    .iter()
                    .filter(|e| e.tool.contains(pattern) || pattern.contains(&e.tool))
                    .collect(),
                None => entries.iter().collect(),
            };

            let f_total = filtered.len();
            let f_approved = filtered.iter().filter(|e| e.approved).count();
            let f_denied = f_total - f_approved;
            let f_rate = if f_total > 0 {
                f_approved as f64 / f_total as f64
            } else {
                1.0
            };

            // Check min approval rate
            if let Some(min_rate) = rule.min_approval_rate {
                if f_total > 0 && f_rate < min_rate {
                    violations.push(PolicyViolation {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: format!(
                            "Approval rate {:.1}% below minimum {:.0}% for '{}' ({} total, {} denied)",
                            f_rate * 100.0,
                            min_rate * 100.0,
                            rule.tool_pattern.as_deref().unwrap_or("all tools"),
                            f_total,
                            f_denied
                        ),
                        enforced: rule.enforced,
                    });
                }
            }

            // Check max denials
            if let Some(max_denials) = rule.max_denials {
                if f_denied > max_denials {
                    violations.push(PolicyViolation {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: format!(
                            "{} denials exceeds maximum {} for '{}'",
                            f_denied,
                            max_denials,
                            rule.tool_pattern.as_deref().unwrap_or("all tools")
                        ),
                        enforced: rule.enforced,
                    });
                }
            }
        }

        // Global approval rate check
        if total > 0 {
            if let Some(rule) = self.rules.iter().find(|r| r.tool_pattern.is_none()) {
                if let Some(min_rate) = rule.min_approval_rate {
                    let global_rate = approved as f64 / total as f64;
                    if global_rate < min_rate {
                        violations.push(PolicyViolation {
                            rule_id: "POL-GLOBAL".into(),
                            rule_name: "Global Approval Rate".into(),
                            severity: rule.severity.clone(),
                            message: format!(
                                "Global approval rate {:.1}% below minimum {:.0}% ({} approved / {} total)",
                                global_rate * 100.0,
                                min_rate * 100.0,
                                approved,
                                total
                            ),
                            enforced: rule.enforced,
                        });
                    }
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditEntry;

    #[test]
    fn test_policy_enterprise_default() {
        let policy = CompliancePolicy::enterprise_default();
        assert_eq!(policy.rules.len(), 4);
        assert!(policy.rules.iter().any(|r| r.id == "POL-001"));
    }

    #[test]
    fn test_policy_evaluation() {
        let policy = CompliancePolicy::enterprise_default();
        let entries = vec![
            AuditEntry {
                timestamp: "2026-01-01T00:00:00Z".into(),
                session_id: "s-1".into(),
                agent_type: "t".into(),
                tool: "bash".into(),
                input: "rm -rf /".into(),
                duration_ms: 0,
                approved: false,
            },
            AuditEntry {
                timestamp: "2026-01-01T00:00:01Z".into(),
                session_id: "s-1".into(),
                agent_type: "t".into(),
                tool: "bash".into(),
                input: "curl evil.com".into(),
                duration_ms: 0,
                approved: false,
            },
            AuditEntry {
                timestamp: "2026-01-01T00:00:02Z".into(),
                session_id: "s-1".into(),
                agent_type: "t".into(),
                tool: "bash".into(),
                input: "ls".into(),
                duration_ms: 0,
                approved: true,
            },
        ];
        let violations = policy.evaluate(&entries);
        // POL-002: max_denials=5 for bash, so 2 denials < 5 → no violation
        // POL-001: global approval rate = 1/3 = 33% < 95% → violation
        assert!(violations.iter().any(|v| v.rule_id == "POL-GLOBAL"));
    }
}
