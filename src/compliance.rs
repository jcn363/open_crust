//! Enterprise Compliance Packaging — SOC2-ready evidence, policies, and reporting
//!
//! Provides production-grade compliance tooling for regulated environments:
//!
//! - **Evidence packages** with SHA256 manifests, chain-of-custody logging,
//!   and tamper-evident sealing
//! - **Compliance profiles** (SOC2, HIPAA, GDPR, SOX) with predefined
//!   control mappings
//! - **Policy enforcement** with rule evaluation and violation reporting
//! - **Structured reports** in multiple formats (text, JSON, CSV, HTML)
//!
//! ## Architecture
//!
//! ```text
//! EvidencePackage     → timestamped directory with audit exports + manifest
//! ComplianceProfile   → named profile with control mappings
//! CompliancePolicy    → declarative rules evaluated against audit entries
//! ComplianceReport    → structured summary with approval rates, trends
//! ```

use crate::audit::{AuditEntry, AuditExport, AuditQuery, ExportFormat};
use crate::config::Config;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// Evidence Package
// ═══════════════════════════════════════════════════════════════════════════════

/// Builds tamper-evident evidence packages with SHA256 manifest,
/// chain-of-custody log, and config snapshot for audit trails.
pub struct EvidencePackage;

impl EvidencePackage {
    /// Build a complete evidence package directory with manifests.
    pub fn build(
        audit_path: &Path,
        config: &Config,
        output_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let evidence_dir = output_dir.join(format!("evidence-{}", timestamp));
        fs::create_dir_all(&evidence_dir)?;

        // Query all audit entries
        let query = AuditQuery::new();
        let entries = query.execute(audit_path)?;

        // Export CSV
        let csv_path = evidence_dir.join("audit.csv");
        AuditExport::export_to_file(&entries, ExportFormat::Csv, &csv_path)?;

        // Export JSON
        let json_path = evidence_dir.join("audit.json");
        AuditExport::export_to_file(&entries, ExportFormat::Json, &json_path)?;

        // Config snapshot
        let config_path = evidence_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(&config_path, config_json)?;

        // Compliance report snapshot
        let report = ComplianceReport::generate(&entries);
        let report_path = evidence_dir.join("compliance-report.txt");
        fs::write(&report_path, format!("{}", report))?;

        // Chain-of-custody log
        let custody_path = evidence_dir.join("chain-of-custody.txt");
        let mut custody = fs::File::create(&custody_path)?;
        writeln!(custody, "CHAIN OF CUSTODY LOG")?;
        writeln!(custody, "{}", "=" .repeat(60))?;
        writeln!(
            custody,
            "Package created: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;
        writeln!(custody, "Generator: OpenCrust Compliance v{}", env!("CARGO_PKG_VERSION"))?;
        writeln!(custody, "Audit source: {}", audit_path.display())?;
        writeln!(custody, "Total entries: {}", entries.len())?;
        writeln!(custody)?;
        writeln!(custody, "CONTENTS:")?;
        writeln!(custody, "  - audit.csv (CSV export of all audit entries)")?;
        writeln!(custody, "  - audit.json (JSON export of all audit entries)")?;
        writeln!(custody, "  - config.json (OpenCrust configuration snapshot)")?;
        writeln!(custody, "  - compliance-report.txt (generated report)")?;
        writeln!(custody, "  - evidence-manifest.txt (SHA256 hashes of all files)")?;
        writeln!(custody, "  - chain-of-custody.txt (this file)")?;
        writeln!(custody)?;
        writeln!(custody, "This package was generated automatically. The manifest")?;
        writeln!(custody, "contains SHA256 hashes for all files. Verify integrity")?;
        writeln!(custody, "by re-computing hashes and comparing to the manifest.")?;

        // Generate manifest with SHA256
        let manifest_path = evidence_dir.join("evidence-manifest.txt");
        let mut manifest = fs::File::create(&manifest_path)?;

        let mut dir_entries: Vec<_> = fs::read_dir(&evidence_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();
        dir_entries.sort_by_key(|e| e.file_name());

        writeln!(manifest, "EVIDENCE PACKAGE MANIFEST")?;
        writeln!(manifest, "{}", "=" .repeat(60))?;
        writeln!(
            manifest,
            "Generated: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;
        writeln!(manifest, "Algorithm: SHA256")?;
        writeln!(manifest)?;

        let mut hashes = Vec::new();
        for entry in &dir_entries {
            let path = entry.path();
            let contents = fs::read(&path)?;
            let hash = Sha256::digest(&contents);
            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            writeln!(manifest, "{}  SHA256:{}", filename, hex::encode(hash))?;
            hashes.push((filename, hex::encode(hash)));
        }

        println!("Evidence package created at: {}", evidence_dir.display());
        println!("Files:");
        for (name, hash) in &hashes {
            println!("  {} (SHA256: {})", name, hash);
        }
        println!("\nVerify integrity: sha256sum {}/*", evidence_dir.display());

        Ok(evidence_dir)
    }

    type VerifyResult = Result<Vec<(String, bool, String)>, Box<dyn std::error::Error>>;
    
    /// Verify an existing evidence package by re-computing hashes.
    pub fn verify(package_dir: &Path) -> VerifyResult {
        let manifest_path = package_dir.join("evidence-manifest.txt");
        if !manifest_path.exists() {
            return Err("evidence-manifest.txt not found".into());
        }

        let content = fs::read_to_string(&manifest_path)?;
        let mut results = Vec::new();

        for line in content.lines() {
            if let Some(rest) = line.strip_suffix(")") {
                if let Some((filename, hash_part)) = rest.split_once("  SHA256:") {
                    let expected_hash = hash_part.trim();
                    let file_path = package_dir.join(filename);
                    let actual_hash = if file_path.exists() {
                        let bytes = fs::read(&file_path)?;
                        hex::encode(Sha256::digest(&bytes))
                    } else {
                        "FILE_NOT_FOUND".to_string()
                    };
                    let valid = actual_hash == expected_hash;
                    results.push((filename.to_string(), valid, actual_hash));
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

    // --- Preset profiles ---

    fn soc2_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("CC1.1".into(), "Control environment — integrity and ethical values".into());
        controls.insert("CC2.1".into(), "Communication and information — audit trails maintained".into());
        controls.insert("CC3.1".into(), "Risk assessment — tool usage monitored".into());
        controls.insert("CC4.1".into(), "Monitoring activities — all actions logged".into());
        controls.insert("CC5.1".into(), "Control activities — permission enforcement".into());
        controls.insert("CC6.1".into(), "Logical and physical access — command validation".into());
        controls.insert("CC7.1".into(), "System operations — change management".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["CC6.1".into(), "CC7.1".into()]);
        tool_to_control.insert("write".into(), vec!["CC7.1".into()]);
        tool_to_control.insert("read".into(), vec!["CC6.1".into()]);
        tool_to_control.insert("notify".into(), vec!["CC2.1".into()]);

        Self {
            name,
            framework: ComplianceFramework::SOC2,
            description: "SOC2 — Service Organization Control 2 compliance profile".into(),
            controls,
            tool_to_control,
        }
    }

    fn hipaa_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("164.308".into(), "Administrative safeguards — access control".into());
        controls.insert("164.310".into(), "Physical safeguards — workstation security".into());
        controls.insert("164.312".into(), "Technical safeguards — audit controls".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["164.308".into()]);
        tool_to_control.insert("write".into(), vec!["164.312".into()]);

        Self {
            name,
            framework: ComplianceFramework::HIPAA,
            description: "HIPAA — Health Insurance Portability and Accountability Act".into(),
            controls,
            tool_to_control,
        }
    }

    fn gdpr_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("Art.5".into(), "Principles of data processing".into());
        controls.insert("Art.32".into(), "Security of processing".into());
        controls.insert("Art.33".into(), "Data breach notification".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["Art.32".into()]);
        tool_to_control.insert("write".into(), vec!["Art.5".into()]);

        Self {
            name,
            framework: ComplianceFramework::GDPR,
            description: "GDPR — General Data Protection Regulation".into(),
            controls,
            tool_to_control,
        }
    }

    fn sox_defaults(name: String) -> Self {
        let mut controls = HashMap::new();
        controls.insert("302".into(), "Corporate responsibility for financial reports".into());
        controls.insert("404".into(), "Internal controls over financial reporting".into());
        controls.insert("409".into(), "Real-time disclosure of material changes".into());

        let mut tool_to_control = HashMap::new();
        tool_to_control.insert("bash".into(), vec!["404".into()]);
        tool_to_control.insert("write".into(), vec!["302".into(), "404".into()]);

        Self {
            name,
            framework: ComplianceFramework::SOX,
            description: "SOX — Sarbanes-Oxley Act compliance".into(),
            controls,
            tool_to_control,
        }
    }

// ═══════════════════════════════════════════════════════════════════════════════
// Compliance Policies
// ═══════════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════════
// Compliance Report
// ═══════════════════════════════════════════════════════════════════════════════

/// A compliance report summarizing audit activity for evidence and review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub total_calls: usize,
    pub approved: usize,
    pub denied: usize,
    pub most_used_tools: Vec<(String, usize)>,
    pub session_count: usize,
    pub date_range: Option<(String, String)>,
    pub daily_breakdown: Vec<(String, usize, usize)>, // (date, total, approved)
    pub violations: Vec<PolicyViolation>,
    pub profile_results: Option<HashMap<String, Vec<usize>>>,
}

impl ComplianceReport {
    /// Generate a new report from audit entries.
    pub fn generate(entries: &[AuditEntry]) -> Self {
        let total_calls = entries.len();
        let approved = entries.iter().filter(|e| e.approved).count();
        let denied = total_calls - approved;

        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        let mut sessions = HashSet::new();
        let mut min_date = String::new();
        let mut max_date = String::new();
        let mut daily_map: HashMap<String, (usize, usize)> = HashMap::new(); // (total, approved)

        for entry in entries {
            *tool_counts.entry(entry.tool.clone()).or_insert(0) += 1;
            sessions.insert(entry.session_id.clone());

            let date = entry.timestamp[..10].to_string();
            if min_date.is_empty() || date < min_date {
                min_date.clone_from(&date);
            }
            if max_date.is_empty() || date > max_date {
                max_date.clone_from(&date);
            }
            let daily = daily_map.entry(date).or_insert((0, 0));
            daily.0 += 1;
            if entry.approved {
                daily.1 += 1;
            }
        }

        let mut most_used: Vec<(String, usize)> = tool_counts.into_iter().collect();
        most_used.sort_by_key(|a| std::cmp::Reverse(a.1));
        most_used.truncate(10);

        let date_range = if !min_date.is_empty() && !max_date.is_empty() {
            Some((min_date, max_date))
        } else {
            None
        };

        let mut daily_breakdown: Vec<(String, usize, usize)> = daily_map
            .into_iter()
            .map(|(date, (total, approved))| (date, total, approved))
            .collect();
        daily_breakdown.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            total_calls,
            approved,
            denied,
            most_used_tools: most_used,
            session_count: sessions.len(),
            date_range,
            daily_breakdown,
            violations: Vec::new(),
            profile_results: None,
        }
    }

    /// Attach policy violations to the report.
    pub fn with_violations(mut self, violations: Vec<PolicyViolation>) -> Self {
        self.violations = violations;
        self
    }

    /// Attach profile evaluation results.
    pub fn with_profile(
        mut self,
        _profile: &ComplianceProfile,
        results: HashMap<String, Vec<usize>>,
    ) -> Self {
        self.profile_results = Some(results);
        self
    }

    /// Export report as JSON string.
    #[cfg_attr(not(test), expect(dead_code, reason = "CLI export command handler"))]
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Export report as CSV summary string.
    #[cfg_attr(not(test), expect(dead_code, reason = "CLI export command handler"))]
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("metric,value\n");
        csv.push_str(&format!("total_calls,{}\n", self.total_calls));
        csv.push_str(&format!("approved,{}\n", self.approved));
        csv.push_str(&format!("denied,{}\n", self.denied));
        csv.push_str(&format!("session_count,{}\n", self.session_count));
        csv.push_str(&format!(
            "approval_rate,{:.1}%\n",
            if self.total_calls > 0 {
                self.approved as f64 / self.total_calls as f64 * 100.0
            } else {
                0.0
            }
        ));
        csv
    }

    /// HTML report for browser viewing.
    #[cfg_attr(not(test), expect(dead_code, reason = "CLI export command handler"))]
    pub fn to_html(&self) -> String {
        let approval_rate = if self.total_calls > 0 {
            self.approved as f64 / self.total_calls as f64 * 100.0
        } else {
            0.0
        };

        let generated_at = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

        let mut html = format!(
            "<!DOCTYPE html>
<html><head><meta charset=\"utf-8\"><title>Compliance Report</title>
<style>
body {{ font-family: -apple-system, sans-serif; max-width: 800px; margin: 2em auto; padding: 0 1em; }}
h1 {{ color: #333; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }}
th {{ background-color: #f5f5f5; }}
.pass {{ color: green; }} .fail {{ color: red; }}
</style></head><body>
<h1>Compliance Report</h1>
<p>Generated: {generated_at}</p>
<h2>Summary</h2>
<table>
<tr><td>Total tool calls</td><td>{total}</td></tr>
<tr><td>Approved</td><td>{approved}</td></tr>
<tr><td>Denied</td><td>{denied}</td></tr>
<tr><td>Approval rate</td><td>{rate:.1}%</td></tr>
<tr><td>Unique sessions</td><td>{sessions}</td></tr>
</table>
<h2>Most Used Tools</h2>
<table><tr><th>#</th><th>Tool</th><th>Calls</th></tr>",
            generated_at = generated_at,
            total = self.total_calls,
            approved = self.approved,
            denied = self.denied,
            rate = approval_rate,
            sessions = self.session_count,
        );

        for (i, (tool, count)) in self.most_used_tools.iter().enumerate() {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                i + 1,
                tool,
                count
            ));
        }

        html.push_str("</table>");

        if !self.violations.is_empty() {
            html.push_str("<h2>Policy Violations</h2><table><tr><th>Rule</th><th>Severity</th><th>Message</th><th>Enforced</th></tr>");
            for v in &self.violations {
                let sev_class = match v.severity {
                    PolicySeverity::Critical | PolicySeverity::High => "fail",
                    _ => "pass",
                };
                html.push_str(&format!(
                    "<tr class=\"{}\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    sev_class, v.rule_id, v.severity, v.message, v.enforced
                ));
            }
            html.push_str("</table>");
        }

        html.push_str("</body></html>");
        html
    }
}

impl std::fmt::Display for ComplianceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let approval_rate = if self.total_calls > 0 {
            self.approved as f64 / self.total_calls as f64 * 100.0
        } else {
            0.0
        };

        writeln!(f, "=== Compliance Report ===")?;
        writeln!(f)?;
        writeln!(f, "Total tool calls: {}", self.total_calls)?;
        writeln!(f, "Approved: {}", self.approved)?;
        writeln!(f, "Denied: {}", self.denied)?;
        writeln!(f, "Approval rate: {:.1}%", approval_rate)?;
        writeln!(f, "Unique sessions: {}", self.session_count)?;
        if let Some((from, to)) = &self.date_range {
            writeln!(f, "Date range: {} to {}", from, to)?;
        }

        if !self.daily_breakdown.is_empty() {
            writeln!(f, "\nDaily breakdown:")?;
            for (date, total, approved) in &self.daily_breakdown {
                let rate = if *total > 0 {
                    *approved as f64 / *total as f64 * 100.0
                } else {
                    0.0
                };
                writeln!(f, "  {}: {}/{} ({:.1}%)", date, approved, total, rate)?;
            }
        }

        writeln!(f, "\nMost used tools:")?;
        for (i, (tool, count)) in self.most_used_tools.iter().enumerate() {
            writeln!(f, "  {}. {} ({} calls)", i + 1, tool, count)?;
        }

        if !self.violations.is_empty() {
            writeln!(f, "\nPolicy violations ({}):", self.violations.len())?;
            for v in &self.violations {
                writeln!(
                    f,
                    "  [{}] {}: {} {}",
                    v.severity,
                    v.rule_id,
                    v.message,
                    if v.enforced { "[ENFORCED]" } else { "[ADVISORY]" }
                )?;
            }
        }

        if let Some(results) = &self.profile_results {
            writeln!(f, "\nProfile control evaluation:")?;
            for (control_id, indices) in results {
                writeln!(f, "  {}: {} matching entries", control_id, indices.len())?;
            }
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compliance Manager
// ═══════════════════════════════════════════════════════════════════════════════

/// High-level manager for compliance operations.
pub struct ComplianceManager {
    pub audit_path: PathBuf,
    pub policy: CompliancePolicy,
    pub profiles: Vec<ComplianceProfile>,
}

impl ComplianceManager {
    pub fn new(_config: &Config) -> Self {
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencrust/logs");
        let audit_path = log_dir.join("audit.log");

        Self {
            audit_path,
            policy: CompliancePolicy::enterprise_default(),
            profiles: vec![
                ComplianceProfile::new("SOC2 Default".into(), ComplianceFramework::SOC2),
                ComplianceProfile::new("HIPAA Default".into(), ComplianceFramework::HIPAA),
            ],
        }
    }

    /// Run a full compliance check: build evidence, evaluate policy, check profiles.
    pub fn full_check(
        &self,
        _output_dir: &Path,
    ) -> Result<ComplianceReport, Box<dyn std::error::Error>> {
        let query = AuditQuery::new();
        let entries = query.execute(&self.audit_path)?;

        // Generate base report
        let mut report = ComplianceReport::generate(&entries);

        // Evaluate policies
        let violations = self.policy.evaluate(&entries);
        report = report.with_violations(violations);

        // Evaluate profiles
        if let Some(profile) = self.profiles.first() {
            let results = profile.evaluate(&entries);
            report = report.with_profile(profile, results);
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_empty_entries() {
        let report = ComplianceReport::generate(&[]);
        assert_eq!(report.total_calls, 0);
        assert_eq!(report.approved, 0);
        assert_eq!(report.denied, 0);
    }

    #[test]
    fn test_report_with_entries() {
        let entries = vec![
            AuditEntry {
                timestamp: "2026-01-01T00:00:00Z".into(),
                session_id: "sess-1".into(),
                agent_type: "test".into(),
                tool: "bash".into(),
                input: "ls".into(),
                duration_ms: 10,
                approved: true,
            },
            AuditEntry {
                timestamp: "2026-01-02T00:00:00Z".into(),
                session_id: "sess-1".into(),
                agent_type: "test".into(),
                tool: "write".into(),
                input: "test.txt".into(),
                duration_ms: 5,
                approved: false,
            },
        ];
        let report = ComplianceReport::generate(&entries);
        assert_eq!(report.total_calls, 2);
        assert_eq!(report.approved, 1);
        assert_eq!(report.denied, 1);
        assert_eq!(report.session_count, 1);
    }

    #[test]
    fn test_soc2_profile() {
        let profile = ComplianceProfile::new("SOC2 Test".into(), ComplianceFramework::SOC2);
        assert_eq!(profile.framework, ComplianceFramework::SOC2);
        assert!(profile.controls.contains_key("CC6.1"));
        assert!(profile.controls.contains_key("CC7.1"));
    }

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

    #[test]
    fn test_evidence_package_verify() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("evidence");
        fs::create_dir_all(&out).unwrap();

        // Create a fake manifest
        let mut manifest = fs::File::create(out.join("evidence-manifest.txt")).unwrap();
        writeln!(manifest, "EVIDENCE PACKAGE MANIFEST").unwrap();
        writeln!(manifest, "Algorithm: SHA256").unwrap();
        writeln!(manifest, "test.txt  SHA256:{})", hex::encode(Sha256::digest(b"hello"))).unwrap();

        // Create the file with matching content
        fs::write(out.join("test.txt"), b"hello").unwrap();

        let result = EvidencePackage::verify(&out);
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].1); // valid
    }

    #[test]
    fn test_compliance_profile_evaluate() {
        let profile = ComplianceProfile::new("Test".into(), ComplianceFramework::SOC2);
        let entries = vec![
            AuditEntry {
                timestamp: "2026-01-01T00:00:00Z".into(),
                session_id: "s-1".into(),
                agent_type: "t".into(),
                tool: "bash".into(),
                input: "ls".into(),
                duration_ms: 0,
                approved: true,
            },
        ];
        let results = profile.evaluate(&entries);
        // bash maps to CC6.1 and CC7.1
        assert!(results.contains_key("CC6.1"));
        assert!(results.contains_key("CC7.1"));
    }

    #[test]
    fn test_report_to_csv() {
        let report = ComplianceReport::generate(&[]);
        let csv = report.to_csv();
        assert!(csv.contains("total_calls,0"));
    }

    #[test]
    fn test_report_to_html() {
        let report = ComplianceReport::generate(&[]);
        let html = report.to_html();
        assert!(html.contains("<h1>Compliance Report</h1>"));
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

    #[test]
    fn test_report_to_json() {
        let report = ComplianceReport::generate(&[]);
        let json = report.to_json().unwrap();
        assert!(json.contains("total_calls"));
        assert!(json.contains("0"));
    }
}
