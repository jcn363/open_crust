//! Compliance profiles and framework definitions

use crate::audit::AuditEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity of a policy violation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for PolicySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicySeverity::Critical => write!(f, "CRITICAL"),
            PolicySeverity::High => write!(f, "HIGH"),
            PolicySeverity::Medium => write!(f, "MEDIUM"),
            PolicySeverity::Low => write!(f, "LOW"),
            PolicySeverity::Info => write!(f, "INFO"),
        }
    }
}

/// A single compliance policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: PolicySeverity,
    /// Tool name pattern to match (glob-like)
    pub tool_pattern: Option<String>,
    /// Max allowed denials for this tool
    pub max_denials: Option<usize>,
    /// Required approval rate (0.0 - 1.0)
    pub min_approval_rate: Option<f64>,
    /// Whether this rule is enforced (vs. advisory)
    pub enforced: bool,
}

/// Result of evaluating a policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: PolicySeverity,
    pub message: String,
    pub enforced: bool,
}

/// Compliance framework standards.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplianceFramework {
    SOC2,
    HIPAA,
    SOX,
    PciDss,
    ISO27001,
}

impl std::fmt::Display for ComplianceFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SOC2 => write!(f, "SOC2"),
            Self::HIPAA => write!(f, "HIPAA"),
            Self::SOX => write!(f, "SOX"),
            Self::PciDss => write!(f, "PCI-DSS"),
            Self::ISO27001 => write!(f, "ISO 27001"),
        }
    }
}

/// A compliance profile for a specific framework (SOC2, HIPAA, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceProfile {
    pub name: String,
    pub framework: ComplianceFramework,
    pub description: String,
    /// Control mappings: control_id → description
    pub controls: HashMap<String, String>,
    /// Which tool patterns map to which controls
    pub tool_to_control: HashMap<String, Vec<String>>,
}

impl ComplianceProfile {
    /// Create a new compliance profile with framework-specific defaults.
    pub fn new(name: String, framework: ComplianceFramework) -> Self {
        match framework {
            ComplianceFramework::SOC2 => Self::soc2_defaults(name),
            ComplianceFramework::HIPAA => Self::hipaa_defaults(name),
            ComplianceFramework::SOX => Self::sox_defaults(name),
            ComplianceFramework::PciDss => Self::pci_dss_defaults(name),
            ComplianceFramework::ISO27001 => Self::iso27001_defaults(name),
        }
    }

    fn soc2_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert(
            "CC1.1".into(),
            "Control environment — integrity and ethical values".into(),
        );
        controls.insert(
            "CC2.1".into(),
            "Communication and information — audit trails maintained".into(),
        );
        controls.insert(
            "CC3.1".into(),
            "Risk assessment — tool usage monitored".into(),
        );
        controls.insert(
            "CC4.1".into(),
            "Monitoring activities — all actions logged".into(),
        );
        controls.insert(
            "CC5.1".into(),
            "Control activities — permission enforcement".into(),
        );
        controls.insert(
            "CC6.1".into(),
            "Logical and physical access — command validation".into(),
        );
        controls.insert(
            "CC7.1".into(),
            "Restricted access — sensitive operations audited".into(),
        );
        controls.insert(
            "CC8.1".into(),
            "Change management — all modifications tracked".into(),
        );
        controls.insert(
            "CC9.1".into(),
            "Incident response — audit logs enable forensics".into(),
        );

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["CC4.1".into(), "CC6.1".into()]);
        tool_to_control.insert("write".into(), vec!["CC5.1".into(), "CC8.1".into()]);
        tool_to_control.insert("read".into(), vec!["CC6.1".into()]);

        Self {
            name,
            framework: ComplianceFramework::SOC2,
            description: "SOC 2 Type II compliance".into(),
            controls,
            tool_to_control,
        }
    }

    fn hipaa_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("164.308(a)(1)".into(), "Security management process".into());
        controls.insert("164.308(a)(3)".into(), "Workforce security".into());
        controls.insert(
            "164.308(a)(4)".into(),
            "Information access management".into(),
        );
        controls.insert("164.312(a)(2)".into(), "Audit controls".into());
        controls.insert("164.312(b)".into(), "Audit logs and accountability".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert(
            "bash".into(),
            vec!["164.312(a)(2)".into(), "164.312(b)".into()],
        );
        tool_to_control.insert("write".into(), vec!["164.308(a)(4)".into()]);

        Self {
            name,
            framework: ComplianceFramework::HIPAA,
            description: "HIPAA compliance".into(),
            controls,
            tool_to_control,
        }
    }

    fn sox_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("IT-1".into(), "IT governance and risk management".into());
        controls.insert("IT-2".into(), "System access and change management".into());
        controls.insert("IT-3".into(), "System monitoring and logging".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["IT-3".into()]);
        tool_to_control.insert("write".into(), vec!["IT-2".into()]);

        Self {
            name,
            framework: ComplianceFramework::SOX,
            description: "SOX — Sarbanes-Oxley Act compliance".into(),
            controls,
            tool_to_control,
        }
    }

    fn pci_dss_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("1.1".into(), "Firewall configuration standards".into());
        controls.insert("2.1".into(), "Default security parameters".into());
        controls.insert("6.1".into(), "Security patches".into());
        controls.insert("7.1".into(), "Access control".into());
        controls.insert("10.1".into(), "Audit logging".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["10.1".into()]);
        tool_to_control.insert("write".into(), vec!["7.1".into()]);

        Self {
            name,
            framework: ComplianceFramework::PciDss,
            description: "PCI DSS compliance".into(),
            controls,
            tool_to_control,
        }
    }

    fn iso27001_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("A.5.1".into(), "Information security policies".into());
        controls.insert("A.6.1".into(), "Internal organization".into());
        controls.insert("A.7.1".into(), "Human resource security".into());
        controls.insert("A.9.1".into(), "Access control".into());
        controls.insert("A.12.4".into(), "Logging".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["A.12.4".into()]);
        tool_to_control.insert("write".into(), vec!["A.9.1".into()]);

        Self {
            name,
            framework: ComplianceFramework::ISO27001,
            description: "ISO 27001 compliance".into(),
            controls,
            tool_to_control,
        }
    }

    /// Evaluate audit entries against this profile.
    /// Returns a map of control_id → list of matching audit entry indices.
    pub fn evaluate(&self, entries: &[AuditEntry]) -> HashMap<String, Vec<usize>> {
        let mut results: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, entry) in entries.iter().enumerate() {
            for (tool_pattern, control_ids) in &self.tool_to_control {
                if entry.tool.contains(tool_pattern) || tool_pattern.contains(&entry.tool) {
                    for cid in control_ids {
                        results.entry(cid.clone()).or_default().push(idx);
                    }
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soc2_profile() {
        let profile = ComplianceProfile::new("SOC2 Test".into(), ComplianceFramework::SOC2);
        assert_eq!(profile.framework, ComplianceFramework::SOC2);
        assert!(profile.controls.contains_key("CC6.1"));
        assert!(profile.controls.contains_key("CC7.1"));
    }

    #[test]
    fn test_compliance_profile_evaluate() {
        let profile = ComplianceProfile::new("Test".into(), ComplianceFramework::SOC2);
        let entries = vec![AuditEntry {
            timestamp: "2026-01-01T00:00:00Z".into(),
            session_id: "s-1".into(),
            agent_type: "t".into(),
            tool: "bash".into(),
            input: "ls".into(),
            duration_ms: 0,
            approved: true,
        }];
        let results = profile.evaluate(&entries);
        // bash maps to CC4.1 and CC6.1 in SOC2 defaults
        assert!(results.contains_key("CC4.1"));
        assert!(results.contains_key("CC6.1"));
    }

    #[test]
    fn test_policy_violation_display() {
        let v = PolicyViolation {
            rule_id: "POL-001".into(),
            rule_name: "Test".into(),
            severity: PolicySeverity::High,
            message: "something is wrong".into(),
            enforced: true,
        };
        let s = format!("{}", v.severity);
        assert_eq!(s, "HIGH");
    }

    #[test]
    fn test_compliance_profile_display() {
        let p = ComplianceProfile::new("test".into(), ComplianceFramework::SOC2);
        let s = format!("{}", p.framework);
        assert_eq!(s, "SOC2");
    }
}
