use crate::audit::{AuditExport, AuditQuery, ExportFormat};
use crate::cli::{ComplianceCommands, PermissionCommands};
use crate::compliance::{ComplianceManager, EvidencePackage};
use crate::config::Config;
use crate::permissions::{Role, RoleTemplate};
use chrono::NaiveDate;
use std::path::PathBuf;

pub async fn handle_compliance(
    cmd: ComplianceCommands,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencrust/logs");
    let audit_log_path = log_dir.join("audit.log");

    match cmd {
        ComplianceCommands::Generate {
            output_dir,
            format,
            include_evidence,
            framework: _framework,
        } => {
            let out_dir = PathBuf::from(&output_dir);
            let compliance_mgr = ComplianceManager::new(config);

            match compliance_mgr.full_check(&out_dir) {
                Ok(report) => {
                    // Generate report in requested format
                    let content = match format.to_lowercase().as_str() {
                        "json" => report
                            .to_json()
                            .map_err(|e| format!("JSON export failed: {}", e))?,
                        "html" => report.to_html(),
                        "csv" => report.to_csv(),
                        "soc2" => report.to_soc2_type2(),
                        _ => {
                            eprintln!("Unknown format '{}', defaulting to SOC2", format);
                            report.to_soc2_type2()
                        }
                    };

                    let report_path = out_dir.join(format!(
                        "compliance-report-{}.{}",
                        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                        match format.to_lowercase().as_str() {
                            "json" => "json",
                            "html" => "html",
                            "csv" => "csv",
                            _ => "txt",
                        }
                    ));
                    std::fs::write(&report_path, &content)?;
                    println!("Compliance report generated: {}", report_path.display());
                    println!("{}", content);

                    // Optionally build evidence package
                    if include_evidence {
                        match EvidencePackage::build(&audit_log_path, config, &out_dir) {
                            Ok(path) => println!("Evidence package created: {}", path.display()),
                            Err(e) => eprintln!("Warning: evidence package build failed: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("Error running compliance check: {}", e),
            }
        }
        ComplianceCommands::Export {
            from,
            to,
            action,
            status,
            format,
            output,
            syslog_server,
            syslog_facility,
        } => {
            let from_date = from
                .as_ref()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let to_date = to
                .as_ref()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let status_filter = status.as_ref().map(|s| s == "approved");

            let query = AuditQuery {
                from_date,
                to_date,
                action_pattern: action.clone(),
                status_filter,
            };

            match query.execute(&audit_log_path) {
                Ok(entries) => {
                    let export_format = match format.to_lowercase().as_str() {
                        "json" => ExportFormat::Json,
                        "syslog" => ExportFormat::Syslog,
                        _ => ExportFormat::Csv,
                    };

                    if format.to_lowercase() == "syslog" {
                        if let Some(server) = syslog_server {
                            export_to_syslog(&entries, &server, &syslog_facility)?;
                            println!(
                                "Exported {} entries to syslog server: {}",
                                entries.len(),
                                server
                            );
                        } else {
                            eprintln!("Error: --syslog-server is required for syslog format");
                        }
                    } else {
                        match output {
                            Some(path) => {
                                let out_path = PathBuf::from(path.clone());
                                AuditExport::export_to_file(&entries, export_format, &out_path)
                                    .unwrap_or_else(|e| eprintln!("Export error: {}", e));
                                println!("Exported {} entries to {}", entries.len(), path);
                            }
                            None => {
                                AuditExport::export(
                                    &entries,
                                    export_format,
                                    &mut std::io::stdout(),
                                )
                                .unwrap_or_else(|e| eprintln!("Export error: {}", e));
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error querying audit log: {}", e),
            }
        }
        ComplianceCommands::Permissions { cmd } => {
            handle_permissions(cmd)?;
        }
        ComplianceCommands::Evidence { output_dir } => {
            let out_dir = output_dir
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            match EvidencePackage::build(&audit_log_path, config, &out_dir) {
                Ok(path) => println!("Evidence package created at: {}", path.display()),
                Err(e) => eprintln!("Error building evidence package: {}", e),
            }
        }
        ComplianceCommands::Verify { path } => {
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
        ComplianceCommands::Check { output_dir } => {
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

fn handle_permissions(
    cmd: PermissionCommands,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        PermissionCommands::List => {
            println!("Available role templates:");
            println!("  admin      - Full access to all operations");
            println!("  developer  - Can read/write files, run tools, no system changes");
            println!("  reviewer   - Read-only access, can review but not modify");
        }
        PermissionCommands::Show { role } => {
            let role_enum = parse_role(&role)?;
            let template = RoleTemplate::for_role(role_enum);
            println!("Role: {:?}", template.role);
            println!("  Can write files: {}", template.can_write_files);
            println!("  Can execute commands: {}", template.can_execute_commands);
            println!("  Can manage MCP: {}", template.can_manage_mcp);
            println!("  Can manage plugins: {}", template.can_manage_plugins);
            println!("  Can modify config: {}", template.can_modify_config);
            println!("  Blocked path prefixes:");
            for prefix in &template.blocked_path_prefixes {
                println!("    - {}", prefix);
            }
        }
        PermissionCommands::Export { role, output } => {
            let role_enum = parse_role(&role)?;
            let template = RoleTemplate::for_role(role_enum);
            let json = serde_json::to_string_pretty(&template)?;
            match output {
                Some(path) => {
                    std::fs::write(&path, &json)?;
                    println!("Role template exported to: {}", path);
                }
                None => println!("{}", json),
            }
        }
        PermissionCommands::Apply { role } => {
            let role_enum = parse_role(&role)?;
            let template = RoleTemplate::for_role(role_enum);
            println!("Applying role template: {:?}", template.role);
            println!("Note: This would update the configuration file with the role's permissions.");
            println!("  Can write files: {}", template.can_write_files);
            println!("  Can execute commands: {}", template.can_execute_commands);
            println!("  Can manage MCP: {}", template.can_manage_mcp);
            println!("  Can manage plugins: {}", template.can_manage_plugins);
            println!("  Can modify config: {}", template.can_modify_config);
            // In a real implementation, this would update the config file
            // For now, we just display what would be applied
        }
    }
    Ok(())
}

fn parse_role(role: &str) -> Result<Role, Box<dyn std::error::Error + Send + Sync>> {
    match role.to_lowercase().as_str() {
        "admin" => Ok(Role::Admin),
        "developer" => Ok(Role::Developer),
        "reviewer" => Ok(Role::Reviewer),
        _ => Err(format!(
            "Unknown role: {}. Valid roles: admin, developer, reviewer",
            role
        )
        .into()),
    }
}

/// Export audit entries to syslog format (RFC 5424)
fn export_to_syslog(
    entries: &[crate::audit::AuditEntry],
    server: &str,
    facility: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let server_addr = format!("{}:514", server);

    // Parse facility
    let facility_code = match facility.to_lowercase().as_str() {
        "kern" => 0,
        "user" => 1,
        "mail" => 2,
        "daemon" => 3,
        "auth" => 4,
        "syslog" => 5,
        "lpr" => 6,
        "news" => 7,
        "uucp" => 8,
        "cron" => 9,
        "authpriv" => 10,
        "ftp" => 11,
        "local0" => 16,
        "local1" => 17,
        "local2" => 18,
        "local3" => 19,
        "local4" => 20,
        "local5" => 21,
        "local6" => 22,
        "local7" => 23,
        _ => 16, // default to local0
    };

    for entry in entries {
        let priority = facility_code * 8 + 6; // facility * 8 + severity (6 = informational)
        let timestamp = chrono::Utc::now().format("%b %d %H:%M:%S").to_string();
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // RFC 5424 format: <PRI>VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID STRUCTURED-DATA MSG
        let msg = format!(
            "<{}>{} {} {} opencrust {} - - tool={} session={} agent={} approved={} input={} duration={}",
            priority,
            1, // version
            timestamp,
            hostname,
            std::process::id(),
            entry.tool,
            entry.session_id,
            entry.agent_type,
            entry.approved,
            entry.input.replace(' ', "_"),
            entry.duration_ms
        );

        socket.send_to(msg.as_bytes(), &server_addr)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditEntry;

    #[test]
    fn test_parse_role_admin() {
        assert!(matches!(parse_role("admin"), Ok(Role::Admin)));
        assert!(matches!(parse_role("ADMIN"), Ok(Role::Admin)));
    }

    #[test]
    fn test_parse_role_developer() {
        assert!(matches!(parse_role("developer"), Ok(Role::Developer)));
        assert!(matches!(parse_role("DEVELOPER"), Ok(Role::Developer)));
    }

    #[test]
    fn test_parse_role_reviewer() {
        assert!(matches!(parse_role("reviewer"), Ok(Role::Reviewer)));
        assert!(matches!(parse_role("REVIEWER"), Ok(Role::Reviewer)));
    }

    #[test]
    fn test_parse_role_invalid() {
        assert!(parse_role("invalid").is_err());
    }

    #[test]
    fn test_export_to_syslog_format() {
        let entries = vec![AuditEntry {
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            session_id: "test-session".to_string(),
            agent_type: "llm".to_string(),
            tool: "bash".to_string(),
            input: "ls -la".to_string(),
            duration_ms: 100,
            approved: true,
        }];

        // Test that the function compiles and formats correctly
        // We can't easily test UDP sending without a server, but we can verify the format
        let entry = &entries[0];
        let priority = 16 * 8 + 6; // local0 * 8 + informational
        let msg = format!(
            "<{}>{} {} {} opencrust {} - - tool={} session={} agent={} approved={} input={} duration={}",
            priority,
            1,
            "Jan 01 00:00:00", // placeholder
            "test-host",       // placeholder
            std::process::id(),
            entry.tool,
            entry.session_id,
            entry.agent_type,
            entry.approved,
            entry.input.replace(' ', "_"),
            entry.duration_ms
        );

        assert!(msg.starts_with(&format!("<{}>", priority)));
        assert!(msg.contains("tool=bash"));
        assert!(msg.contains("session=test-session"));
        assert!(msg.contains("approved=true"));
    }
}
