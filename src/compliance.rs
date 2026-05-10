#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::audit::{AuditEntry, AuditExport, AuditQuery, ExportFormat};
use crate::config::Config;

/// Builds an evidence package: a timestamped directory with audit exports,
/// config snapshot, and a SHA256 manifest for audit trail integrity.
pub struct EvidencePackage;

impl EvidencePackage {
    pub fn build(
        audit_path: &Path,
        config: &Config,
        output_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        // Generate manifest with SHA256
        let manifest_path = evidence_dir.join("evidence-manifest.txt");
        let mut manifest = fs::File::create(&manifest_path)?;

        let mut dir_entries: Vec<_> = fs::read_dir(&evidence_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();
        dir_entries.sort_by_key(|e| e.file_name());

        let mut hashes = Vec::new();
        for entry in &dir_entries {
            let path = entry.path();
            let contents = fs::read(&path)?;
            let hash = Sha256::digest(&contents);
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            writeln!(manifest, "{}  SHA256:{}", filename, hex::encode(hash))?;
            hashes.push((filename, hex::encode(hash)));
        }

        println!("Evidence package created at: {}", evidence_dir.display());
        println!("Files:");
        for (name, hash) in &hashes {
            println!("  {} (SHA256: {})", name, hash);
        }

        Ok(())
    }
}

/// A compliance report summarizing audit activity for evidence and review.
pub struct ComplianceReport {
    pub total_calls: usize,
    pub approved: usize,
    pub denied: usize,
    pub most_used_tools: Vec<(String, usize)>,
    pub session_count: usize,
    pub date_range: Option<(String, String)>,
}

impl ComplianceReport {
    pub fn generate(entries: &[AuditEntry]) -> Self {
        let total_calls = entries.len();
        let approved = entries.iter().filter(|e| e.approved).count();
        let denied = total_calls - approved;

        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        let mut sessions = HashSet::new();
        let mut min_date = String::new();
        let mut max_date = String::new();

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
        }

        let mut most_used: Vec<(String, usize)> = tool_counts.into_iter().collect();
        most_used.sort_by_key(|a| std::cmp::Reverse(a.1));
        most_used.truncate(10);

        let date_range = if !min_date.is_empty() && !max_date.is_empty() {
            Some((min_date, max_date))
        } else {
            None
        };

        Self {
            total_calls,
            approved,
            denied,
            most_used_tools: most_used,
            session_count: sessions.len(),
            date_range,
        }
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
        writeln!(f, "\nMost used tools:")?;
        for (i, (tool, count)) in self.most_used_tools.iter().enumerate() {
            writeln!(f, "  {}. {} ({} calls)", i + 1, tool, count)?;
        }
        Ok(())
    }
}
