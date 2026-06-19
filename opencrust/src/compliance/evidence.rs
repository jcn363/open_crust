//! Evidence package building and verification

use crate::audit::{AuditExport, AuditQuery, ExportFormat};
use crate::config::Config;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::reports::ComplianceReport;

/// Result type for evidence package verification operations.
pub type VerifyResult = Result<Vec<(String, bool, String)>, Box<dyn std::error::Error>>;

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
        writeln!(custody, "{}", "=".repeat(60))?;
        writeln!(
            custody,
            "Package created: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;
        writeln!(
            custody,
            "Generator: OpenCrust Compliance v{}",
            env!("CARGO_PKG_VERSION")
        )?;
        writeln!(custody, "Audit source: {}", audit_path.display())?;
        writeln!(custody, "Total entries: {}", entries.len())?;
        writeln!(custody)?;
        writeln!(custody, "CONTENTS:")?;
        writeln!(custody, "  - audit.csv (CSV export of all audit entries)")?;
        writeln!(custody, "  - audit.json (JSON export of all audit entries)")?;
        writeln!(
            custody,
            "  - config.json (OpenCrust configuration snapshot)"
        )?;
        writeln!(custody, "  - compliance-report.txt (generated report)")?;
        writeln!(
            custody,
            "  - evidence-manifest.txt (SHA256 hashes of all files)"
        )?;
        writeln!(custody, "  - chain-of-custody.txt (this file)")?;
        writeln!(custody)?;
        writeln!(
            custody,
            "This package was generated automatically. The manifest"
        )?;
        writeln!(
            custody,
            "contains SHA256 hashes for all files. Verify integrity"
        )?;
        writeln!(
            custody,
            "by re-computing hashes and comparing to the manifest."
        )?;

        // Generate manifest with SHA256
        let manifest_path = evidence_dir.join("evidence-manifest.txt");
        let mut manifest = fs::File::create(&manifest_path)?;

        let mut dir_entries: Vec<_> = fs::read_dir(&evidence_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();
        dir_entries.sort_by_key(|e| e.file_name());

        writeln!(manifest, "EVIDENCE PACKAGE MANIFEST")?;
        writeln!(manifest, "{}", "=".repeat(60))?;
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
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_evidence_package_verify() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("evidence");
        fs::create_dir_all(&out).unwrap();

        // Create a fake manifest
        let mut manifest = fs::File::create(out.join("evidence-manifest.txt")).unwrap();
        writeln!(manifest, "EVIDENCE PACKAGE MANIFEST").unwrap();
        writeln!(manifest, "Algorithm: SHA256").unwrap();
        writeln!(
            manifest,
            "test.txt  SHA256:{})",
            hex::encode(Sha256::digest(b"hello"))
        )
        .unwrap();

        // Create the file with matching content
        fs::write(out.join("test.txt"), b"hello").unwrap();

        let result = EvidencePackage::verify(&out);
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].1); // valid
    }
}
