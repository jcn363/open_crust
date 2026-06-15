//! Compliance reporting — structured summaries with approval rates and trends

use crate::audit::{AuditEntry, AuditQuery};
use crate::config::Config;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::policies::CompliancePolicy;
use super::profiles::{ComplianceFramework, ComplianceProfile, PolicySeverity, PolicyViolation};

/// A compliance report summarizing audit activity for evidence and review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub total_calls: usize,
    pub approved: usize,
    pub denied: usize,
    pub most_used_tools: Vec<(String, usize)>,
    pub session_count: usize,
    pub date_range: Option<(String, String)>,
    pub daily_breakdown: Vec<(String, usize, usize)>,
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
        let mut daily_map: HashMap<String, (usize, usize)> = HashMap::new();

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
                    if v.enforced {
                        "[ENFORCED]"
                    } else {
                        "[ADVISORY]"
                    }
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

        let mut report = ComplianceReport::generate(&entries);

        let violations = self.policy.evaluate(&entries);
        report = report.with_violations(violations);

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
    fn test_report_to_json() {
        let report = ComplianceReport::generate(&[]);
        let json = report.to_json().unwrap();
        assert!(json.contains("total_calls"));
        assert!(json.contains("0"));
    }
}
