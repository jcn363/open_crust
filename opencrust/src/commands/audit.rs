use crate::audit::AuditQuery;
use crate::cli::AuditCommands;
use crate::compliance::{ComplianceManager, EvidencePackage};
use crate::config::Config;
use std::path::PathBuf;

pub async fn handle_audit(
    cmd: AuditCommands,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencrust/logs");
    let audit_log_path = log_dir.join("audit.log");

    match cmd {
        AuditCommands::Export {
            from,
            to,
            action,
            status,
            format,
            output,
        } => {
            let from_date = from
                .as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let to_date = to
                .as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let status_filter = status.as_ref().map(|s| s == "approved");

            let query = AuditQuery {
                from_date,
                to_date,
                action_pattern: action.clone(),
                status_filter,
            };

            match query.execute(&audit_log_path) {
                Ok(entries) => {
                    let fmt = match format.as_str() {
                        "json" => crate::audit::ExportFormat::Json,
                        _ => crate::audit::ExportFormat::Csv,
                    };

                    match output {
                        Some(path) => {
                            let out_path = PathBuf::from(path.clone());
                            crate::audit::AuditExport::export_to_file(&entries, fmt, &out_path)
                                .unwrap_or_else(|e| eprintln!("Export error: {}", e));
                            println!("Exported {} entries to {}", entries.len(), path);
                        }
                        None => {
                            crate::audit::AuditExport::export(
                                &entries,
                                fmt,
                                &mut std::io::stdout(),
                            )
                            .unwrap_or_else(|e| eprintln!("Export error: {}", e));
                        }
                    }
                }
                Err(e) => eprintln!("Error querying audit log: {}", e),
            }
        }
        AuditCommands::Query {
            from,
            to,
            action,
            status,
        } => {
            let from_date = from
                .as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let to_date = to
                .as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let status_filter = status.as_ref().map(|s| s == "approved");

            let query = AuditQuery {
                from_date,
                to_date,
                action_pattern: action.clone(),
                status_filter,
            };

            match query.execute(&audit_log_path) {
                Ok(entries) => {
                    if entries.is_empty() {
                        println!("No matching audit entries found.");
                    } else {
                        println!(
                            "{:<26} {:<16} {:<10} {:<20} {:<8}",
                            "Timestamp", "Session", "Agent", "Tool", "Status"
                        );
                        println!("{}", "-".repeat(80));
                        for entry in &entries {
                            let status_str = if entry.approved { "APPROVED" } else { "DENIED" };
                            println!(
                                "{:<26} {:<16} {:<10} {:<20} {:<8}",
                                entry.timestamp,
                                entry.session_id.chars().take(14).collect::<String>(),
                                entry.agent_type.chars().take(8).collect::<String>(),
                                entry.tool.chars().take(18).collect::<String>(),
                                status_str,
                            );
                        }
                        println!("\nTotal: {} entries", entries.len());
                    }
                }
                Err(e) => eprintln!("Error querying audit log: {}", e),
            }
        }
        AuditCommands::Evidence { output_dir } => {
            let out_dir = output_dir
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            match EvidencePackage::build(&audit_log_path, config, &out_dir) {
                Ok(path) => println!("Evidence package created at: {}", path.display()),
                Err(e) => eprintln!("Error building evidence package: {}", e),
            }
        }
        AuditCommands::Report { from, to } => {
            let from_date = from
                .as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let to_date = to
                .as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

            let query = AuditQuery {
                from_date,
                to_date,
                action_pattern: None,
                status_filter: None,
            };

            match query.execute(&audit_log_path) {
                Ok(entries) => {
                    let report = crate::compliance::ComplianceReport::generate(&entries);
                    println!("{}", report);
                }
                Err(e) => eprintln!("Error generating report: {}", e),
            }
        }
        AuditCommands::Policy { output_dir } => {
            let out_dir = output_dir
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let compliance_mgr = ComplianceManager::new(config);
            match compliance_mgr.full_check(&out_dir) {
                Ok(report) => {
                    println!("{}", report);
                    if !report.violations.is_empty() {
                        println!("\nPolicy violations found: {}", report.violations.len());
                        for v in &report.violations {
                            println!("  [{}] {}: {}", v.severity, v.rule_id, v.message);
                        }
                    } else {
                        println!("\nNo policy violations found.");
                    }
                }
                Err(e) => eprintln!("Error running compliance check: {}", e),
            }
        }
        AuditCommands::Verify { path } => {
            let pkg_dir = PathBuf::from(&path);
            if !pkg_dir.exists() {
                eprintln!("Error: evidence package directory '{}' not found", path);
                return Ok(());
            }
            match EvidencePackage::verify(&pkg_dir) {
                Ok(results) => {
                    println!("Verification results for: {}", pkg_dir.display());
                    let mut all_valid = true;
                    for (name, valid, hash) in &results {
                        let status = if *valid { "VALID" } else { "MISMATCH" };
                        if !*valid {
                            all_valid = false;
                        }
                        println!("  {}: {} ({})", name, status, hash);
                    }
                    if all_valid {
                        println!("\n✓ All files verified successfully.");
                    } else {
                        println!("\n✗ Some files failed verification!");
                    }
                }
                Err(e) => eprintln!("Error verifying evidence package: {}", e),
            }
        }
        AuditCommands::Check { output_dir } => {
            let out_dir = PathBuf::from(&output_dir);
            let compliance_mgr = ComplianceManager::new(config);
            match compliance_mgr.full_check(&out_dir) {
                Ok(report) => {
                    println!("=== Full Compliance Check ===");
                    println!("{}", report);
                    if report.violations.is_empty() {
                        println!("\n✓ All compliance checks passed.");
                    } else {
                        println!("\n✗ {} policy violation(s) found.", report.violations.len());
                    }
                    // Also build evidence package
                    match EvidencePackage::build(&audit_log_path, config, &out_dir) {
                        Ok(path) => println!("\nEvidence package: {}", path.display()),
                        Err(e) => eprintln!("\nWarning: evidence package build failed: {}", e),
                    }
                }
                Err(e) => eprintln!("Error running compliance check: {}", e),
            }
        }
    }
    Ok(())
}
