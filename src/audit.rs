//! Audit logging and query system
//!
//! Provides structured audit logging of tool executions with timestamps,
//! session IDs, and approval status. Supports CSV/JSON export, log rotation,
//! retention-based cleanup, and filtering queries for compliance use cases.

use chrono::{Local, NaiveDate, Utc};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Structured audit logger for tool executions
///
/// Writes timestamped log entries with session ID, agent type, tool name,
/// input, duration, and approval status. Supports log rotation, retention
/// cleanup, and compliance mode for regulated environments.
pub struct AuditLogger {
    log_path: PathBuf,
    session_id: String,
    agent_type: Option<String>,
    max_size_bytes: u64,
    compliance_mode: bool,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self::with_max_size(10_485_760)
    }

    pub fn with_max_size(max_size_bytes: u64) -> Self {
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencrust/logs");

        if !log_dir.exists() {
            let _ = fs::create_dir_all(&log_dir);
        }

        Self {
            log_path: log_dir.join("audit.log"),
            session_id: String::new(),
            agent_type: None,
            max_size_bytes,
            compliance_mode: false,
        }
    }

    #[expect(dead_code, reason = "AuditLogger builder API")]
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = session_id;
        self
    }

    #[expect(dead_code, reason = "AuditLogger builder API")]
    pub fn with_agent_type(mut self, agent_type: Option<String>) -> Self {
        self.agent_type = agent_type;
        self
    }

    #[expect(dead_code, reason = "AuditLogger builder API")]
    pub fn with_compliance_mode(mut self, enabled: bool) -> Self {
        self.compliance_mode = enabled;
        self
    }

    pub fn log_action(&self, tool_name: &str, input: &str, approved: bool) {
        self.log_action_with_duration(tool_name, input, approved, 0);
    }

    pub fn log_action_with_duration(
        &self,
        tool_name: &str,
        input: &str,
        approved: bool,
        duration_ms: u64,
    ) {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let status = if approved { "APPROVED" } else { "DENIED" };
        let session = &self.session_id;
        let agent = self.agent_type.as_deref().unwrap_or("unknown");
        let safe_input = redact_sensitive_data(tool_name, input);
        let log_entry = format!(
            "[{}] session={} agent={} tool={} input={} duration={} status={}\n",
            timestamp, session, agent, tool_name, safe_input, duration_ms, status,
        );

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(log_entry.as_bytes()) {
                    eprintln!("Warning: Failed to write audit log: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to open audit log: {}", e);
            }
        }

        self.check_rotation();
    }

    pub fn check_rotation(&self) {
        // In compliance mode, logs must never be rotated (append-only audit trail)
        if self.compliance_mode {
            return;
        }
        if let Ok(metadata) = fs::metadata(&self.log_path)
            && metadata.len() > self.max_size_bytes
        {
            let timestamp = Local::now().format("%Y-%m-%d_%H%M%S").to_string();
            let rotated_path = self
                .log_path
                .with_file_name(format!("audit.{}.log", timestamp));
            let _ = fs::rename(&self.log_path, &rotated_path);
            let _ = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&self.log_path);
        }
    }

    #[expect(dead_code, reason = "AuditLogger maintenance API")]
    pub fn cleanup_old_logs(&self, retention_days: u64) {
        // In compliance mode, logs must never be deleted (immutable audit trail)
        if self.compliance_mode {
            return;
        }
        let log_dir = self.log_path.parent().unwrap_or(Path::new("."));
        let Ok(entries) = fs::read_dir(log_dir) else {
            return;
        };
        let cutoff = Local::now() - chrono::Duration::days(retention_days as i64);
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !file_name.starts_with("audit.") || !file_name.ends_with(".log") {
                continue;
            }
            let Ok(metadata) = fs::metadata(&path) else {
                continue;
            };
            let Ok(modified) = metadata.modified() else {
                continue;
            };
            let datetime: chrono::DateTime<Local> = modified.into();
            if datetime < cutoff {
                let _ = fs::remove_file(&path);
            }
        }
    }
}

pub struct AuditEntry {
    pub timestamp: String,
    pub session_id: String,
    pub agent_type: String,
    pub tool: String,
    pub input: String,
    pub duration_ms: u64,
    pub approved: bool,
}

impl AuditEntry {
    /// Parse a log entry from a line of text
    pub fn parse_entry(line: &str) -> Option<AuditEntry> {
        let line = line.trim();
        if !line.starts_with('[') {
            return None;
        }
        let close_bracket = line.find(']')?;
        let timestamp = line[1..close_bracket].to_string();

        // Validate timestamp format (at least "YYYY-MM-DD")
        if timestamp.len() < 10 {
            return None;
        }

        let rest = line[close_bracket + 1..].trim();
        let mut session_id = String::new();
        let mut agent_type = String::new();
        let mut tool = String::new();
        let mut input = String::new();
        let mut duration_ms: u64 = 0;
        let mut approved = false;

        for part in rest.split_whitespace() {
            if let Some(val) = part.strip_prefix("session=") {
                session_id = val.to_string();
            } else if let Some(val) = part.strip_prefix("agent=") {
                agent_type = val.to_string();
            } else if let Some(val) = part.strip_prefix("tool=") {
                tool = val.to_string();
            } else if let Some(val) = part.strip_prefix("input=") {
                input = val.to_string();
            } else if let Some(val) = part.strip_prefix("duration=") {
                duration_ms = val.parse().unwrap_or(0);
            } else if let Some(val) = part.strip_prefix("status=") {
                approved = val == "APPROVED";
            }
        }

        Some(AuditEntry {
            timestamp,
            session_id,
            agent_type,
            tool,
            input,
            duration_ms,
            approved,
        })
    }
}

pub struct AuditQuery {
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub action_pattern: Option<String>,
    pub status_filter: Option<bool>,
}

impl AuditQuery {
    pub fn new() -> Self {
        Self {
            from_date: None,
            to_date: None,
            action_pattern: None,
            status_filter: None,
        }
    }

    #[expect(dead_code, reason = "AuditQuery builder API")]
    pub fn with_dates(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Self {
        Self {
            from_date: from,
            to_date: to,
            action_pattern: None,
            status_filter: None,
        }
    }

    #[expect(dead_code, reason = "AuditQuery builder API")]
    pub fn with_action(pattern: Option<String>) -> Self {
        Self {
            from_date: None,
            to_date: None,
            action_pattern: pattern,
            status_filter: None,
        }
    }

    #[expect(dead_code, reason = "AuditQuery builder API")]
    pub fn with_status(status: Option<bool>) -> Self {
        Self {
            from_date: None,
            to_date: None,
            action_pattern: None,
            status_filter: status,
        }
    }
    fn matches(&self, entry: &AuditEntry) -> bool {
        if let Some(from) = self.from_date {
            if entry.timestamp.len() < 10 {
                return true; // Include entries with malformed timestamps in date-filtered queries
            }
            if let Ok(entry_date) = NaiveDate::parse_from_str(&entry.timestamp[..10], "%Y-%m-%d")
                && entry_date < from
            {
                return false;
            }
        }
        if let Some(to) = self.to_date {
            if entry.timestamp.len() < 10 {
                return true;
            }
            if let Ok(entry_date) = NaiveDate::parse_from_str(&entry.timestamp[..10], "%Y-%m-%d")
                && entry_date > to
            {
                return false;
            }
        }
        if let Some(ref pattern) = self.action_pattern
            && !entry.tool.contains(pattern)
        {
            return false;
        }
        if let Some(status) = self.status_filter
            && entry.approved != status
        {
            return false;
        }
        true
    }

    pub fn execute(&self, log_path: &Path) -> Result<Vec<AuditEntry>, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();
        let log_dir = log_path.parent().unwrap_or(Path::new("."));

        let mut log_files: Vec<PathBuf> = Vec::new();
        if log_path.exists() {
            log_files.push(log_path.to_path_buf());
        }
        if let Ok(dir_entries) = fs::read_dir(log_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if name.starts_with("audit.") && name.ends_with(".log") && path != *log_path {
                    log_files.push(path);
                }
            }
        }

        for file_path in &log_files {
            if let Ok(file) = fs::File::open(file_path) {
                let reader = std::io::BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    if let Some(entry) = AuditEntry::parse_entry(&line)
                        && self.matches(&entry)
                    {
                        entries.push(entry);
                    }
                }
            }
        }

        Ok(entries)
    }
}

pub enum ExportFormat {
    Csv,
    Json,
}

pub struct AuditExport;

impl AuditExport {
    pub fn export(
        entries: &[AuditEntry],
        format: ExportFormat,
        writer: &mut dyn Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match format {
            ExportFormat::Csv => {
                writeln!(
                    writer,
                    "timestamp,session_id,agent_type,tool,input,duration_ms,approved"
                )?;
                for entry in entries {
                    writeln!(
                        writer,
                        "{},{},{},{},{},{},{}",
                        entry.timestamp,
                        entry.session_id,
                        entry.agent_type,
                        entry.tool,
                        entry.input,
                        entry.duration_ms,
                        entry.approved,
                    )?;
                }
            }
            ExportFormat::Json => {
                let json_entries: Vec<serde_json::Value> = entries
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "timestamp": e.timestamp,
                            "session_id": e.session_id,
                            "agent_type": e.agent_type,
                            "tool": e.tool,
                            "input": e.input,
                            "duration_ms": e.duration_ms,
                            "approved": e.approved,
                        })
                    })
                    .collect();
                let json_str = serde_json::to_string_pretty(&json_entries)?;
                writeln!(writer, "{}", json_str)?;
            }
        }
        Ok(())
    }

    pub fn export_to_file(
        entries: &[AuditEntry],
        format: ExportFormat,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::create(path)?;
        Self::export(entries, format, &mut file)
    }
}

/// Redact sensitive data from audit logs.
///
/// Redacts:
/// - API keys, tokens, passwords, secrets (key=value patterns)
/// - File contents for read/write tools (logs only path)
/// - Command arguments that may contain secrets
/// - Environment variables that may contain credentials
fn redact_sensitive_data(tool_name: &str, input: &str) -> String {
    // For file read/write tools, only log the path, not the content
    if matches!(
        tool_name,
        "read" | "write" | "edit" | "bash" | "glob" | "grep" | "ls"
    ) {
        // Try to extract just the path from the input
        if let Some(path) = extract_path_from_input(input) {
            return format!("path={}", path);
        }
    }

    let mut redacted = input.to_string();

    // Redact key=value patterns where key suggests sensitive data
    let sensitive_keys = [
        "api_key",
        "apikey",
        "api-key",
        "token",
        "password",
        "passwd",
        "secret",
        "access_token",
        "access-key",
        "access_key",
        "auth_token",
        "auth-token",
        "auth_key",
        "authkey",
        "private_key",
        "private-key",
        "privatekey",
        "secret_key",
        "secret-key",
        "secretkey",
        "bearer",
        "authorization",
        "x-api-key",
        "x-auth-token",
        "api_secret",
        "api-secret",
        "client_secret",
        "client-secret",
        "client_id",
        "client-id",
        "refresh_token",
        "refresh-token",
    ];

    for key in &sensitive_keys {
        // Match key=value, key="value", key='value', key: value patterns
        let patterns = [
            format!("{}=", key),
            format!("{}=\"", key),
            format!("{}='", key),
            format!("{}: ", key),
            format!("{}: \"", key),
            format!("{}: '", key),
        ];

        for pattern in &patterns {
            if let Some(pos) = redacted.find(pattern) {
                let start = pos + pattern.len();
                // Find the end of the value (space, comma, }, ", ', or end of string)
                let end = redacted[start..]
                    .find(|c: char| {
                        c.is_whitespace() || c == ',' || c == '}' || c == '"' || c == '\''
                    })
                    .map(|e| start + e)
                    .unwrap_or(redacted.len());
                if end > start {
                    redacted.replace_range(start..end, "[REDACTED]");
                }
            }
        }
    }

    // Redact common token patterns (long alphanumeric strings that look like tokens)
    // JWT tokens (three base64 parts separated by dots)
    redacted = redact_jwt_tokens(&redacted);

    // Redact Bearer tokens in Authorization headers
    if let Some(pos) = redacted.find("Bearer ") {
        let start = pos + "Bearer ".len();
        let end = redacted[start..]
            .find(|c: char| c.is_whitespace() || c == ',' || c == '}')
            .map(|e| start + e)
            .unwrap_or(redacted.len());
        if end > start {
            redacted.replace_range(start..end, "[REDACTED]");
        }
    }

    // Replace newlines and carriage returns with spaces for log readability
    redacted = redacted.replace(['\n', '\r'], " ");

    redacted
}

/// Extract a file path from tool input for logging purposes
fn extract_path_from_input(input: &str) -> Option<String> {
    // Try to parse as JSON first (many tools use JSON input)
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
        // Common path fields in tool inputs
        for field in [
            "path",
            "file_path",
            "filepath",
            "filename",
            "file",
            "dir",
            "directory",
        ] {
            if let Some(path) = json.get(field).and_then(|v| v.as_str()) {
                return Some(path.to_string());
            }
        }
        // For bash commands, try to extract the first argument that looks like a path
        if let Some(cmd) = json.get("command").and_then(|v| v.as_str()) {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.len() > 1 {
                // Return the first non-flag argument
                for part in &parts[1..] {
                    if !part.starts_with('-')
                        && (part.contains('/')
                            || part.contains('.')
                            || part.starts_with("./")
                            || part.starts_with("../"))
                    {
                        return Some(part.to_string());
                    }
                }
            }
        }
    }

    // Fallback: if input looks like a simple path
    let trimmed = input.trim();
    if (trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../"))
        && !trimmed.contains(' ')
        && trimmed.len() < 256
    {
        return Some(trimmed.to_string());
    }

    None
}

/// Redact JWT tokens (three base64 parts separated by dots)
fn redact_jwt_tokens(input: &str) -> String {
    let mut result = input.to_string();
    // Simple regex-like matching for JWT pattern: xxxxx.yyyyy.zzzzz
    // Collect potential JWT tokens first to avoid borrow checker issues
    let potential_tokens: Vec<String> = result
        .split_whitespace()
        .filter(|part| part.matches('.').count() == 2)
        .map(|s| s.to_string())
        .collect();

    for part in potential_tokens {
        let segments: Vec<&str> = part.split('.').collect();
        if segments.len() == 3
            && segments.iter().all(|s| {
                !s.is_empty()
                    && s.chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            })
        {
            result = result.replace(&part, "[REDACTED_JWT]");
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_entry_valid() {
        let line = "[2026-05-12T10:30:00.000Z] session=abc123 agent=llm tool=bash input=ls duration=100 status=APPROVED";
        let entry = AuditEntry::parse_entry(line).unwrap();
        assert_eq!(entry.timestamp, "2026-05-12T10:30:00.000Z");
        assert_eq!(entry.session_id, "abc123");
        assert_eq!(entry.tool, "bash");
        assert!(entry.approved);
    }

    #[test]
    fn test_parse_entry_missing_brackets() {
        let line = "no brackets here";
        let entry = AuditEntry::parse_entry(line);
        assert!(entry.is_none());
    }

    #[test]
    fn test_parse_entry_short_timestamp() {
        // Timestamp too short (< 10 chars) should return None
        let line = "[2026-05-1] session=abc123 tool=ls";
        let entry = AuditEntry::parse_entry(line);
        assert!(entry.is_none());
    }

    #[test]
    fn test_parse_entry_malformed() {
        let line = "[2026-05-12T10:30:00Z]";
        let entry = AuditEntry::parse_entry(line);
        assert!(entry.is_some()); // Should parse with empty fields
        let e = entry.unwrap();
        assert_eq!(e.session_id, "");
        assert_eq!(e.tool, "");
    }

    #[test]
    fn test_matches_date_filter() {
        let query = AuditQuery {
            from_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            to_date: Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            action_pattern: None,
            status_filter: None,
        };
        let entry = AuditEntry {
            timestamp: "2026-06-15T10:00:00.000Z".to_string(),
            session_id: "s1".to_string(),
            agent_type: "llm".to_string(),
            tool: "bash".to_string(),
            input: "ls".to_string(),
            duration_ms: 0,
            approved: true,
        };
        assert!(query.matches(&entry));

        let entry_outside = AuditEntry {
            timestamp: "2025-06-15T10:00:00.000Z".to_string(),
            session_id: "s1".to_string(),
            agent_type: "llm".to_string(),
            tool: "bash".to_string(),
            input: "ls".to_string(),
            duration_ms: 0,
            approved: true,
        };
        assert!(!query.matches(&entry_outside));
    }

    #[test]
    fn test_export_csv() {
        let entries = vec![AuditEntry {
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            session_id: "test-session".to_string(),
            agent_type: "llm".to_string(),
            tool: "bash".to_string(),
            input: "ls -la".to_string(),
            duration_ms: 100,
            approved: true,
        }];
        let mut output = Vec::new();
        let result = AuditExport::export(&entries, ExportFormat::Csv, &mut output);
        assert!(result.is_ok());
        let csv = String::from_utf8(output).unwrap();
        assert!(csv.contains("timestamp"));
        assert!(csv.contains("test-session"));
    }

    #[test]
    fn test_export_json() {
        let entries = vec![AuditEntry {
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            session_id: "test-session".to_string(),
            agent_type: "llm".to_string(),
            tool: "bash".to_string(),
            input: "ls -la".to_string(),
            duration_ms: 100,
            approved: true,
        }];
        let mut output = Vec::new();
        let result = AuditExport::export(&entries, ExportFormat::Json, &mut output);
        assert!(result.is_ok());
        let json = String::from_utf8(output).unwrap();
        assert!(json.contains("test-session"));
    }

    #[test]
    fn test_export_to_file() {
        let entries = vec![AuditEntry {
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            session_id: "test-session".to_string(),
            agent_type: "llm".to_string(),
            tool: "bash".to_string(),
            input: "ls -la".to_string(),
            duration_ms: 100,
            approved: true,
        }];
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        drop(file);
        let result = AuditExport::export_to_file(&entries, ExportFormat::Csv, &path);
        assert!(result.is_ok());
        assert!(path.exists());
    }
}
