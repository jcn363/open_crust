#![allow(dead_code)]

use chrono::{NaiveDate, Local, Utc};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub struct AuditLogger {
    log_path: PathBuf,
    session_id: String,
    agent_type: Option<String>,
    max_size_bytes: u64,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self::with_max_size(10_485_760)
    }

    pub fn with_max_size(max_size_bytes: u64) -> Self {
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("open_crust/logs");

        if !log_dir.exists() {
            let _ = fs::create_dir_all(&log_dir);
        }

        Self {
            log_path: log_dir.join("audit.log"),
            session_id: String::new(),
            agent_type: None,
            max_size_bytes,
        }
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_agent_type(mut self, agent_type: Option<String>) -> Self {
        self.agent_type = agent_type;
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
        let log_entry = format!(
            "[{}] session={} agent={} tool={} input={} duration={} status={}\n",
            timestamp, session, agent, tool_name, input, duration_ms, status,
        );

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }

        self.check_rotation();
    }

    pub fn check_rotation(&self) {
        if let Ok(metadata) = fs::metadata(&self.log_path) {
            if metadata.len() > self.max_size_bytes {
                let date_str = Local::now().format("%Y-%m-%d").to_string();
                let rotated_path = self.log_path.with_file_name(format!("audit.{}.log", date_str));
                let _ = fs::rename(&self.log_path, &rotated_path);
                let _ = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&self.log_path);
            }
        }
    }

    pub fn cleanup_old_logs(&self, retention_days: u64) {
        let log_dir = self.log_path.parent().unwrap_or(Path::new("."));
        if let Ok(entries) = fs::read_dir(log_dir) {
            let cutoff = Local::now() - chrono::Duration::days(retention_days as i64);
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("audit.") && file_name.ends_with(".log") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                let datetime: chrono::DateTime<Local> = modified.into();
                                if datetime < cutoff {
                                    let _ = fs::remove_file(&path);
                                }
                            }
                        }
                    }
                }
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

    pub fn with_dates(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Self {
        Self {
            from_date: from,
            to_date: to,
            action_pattern: None,
            status_filter: None,
        }
    }

    pub fn with_action(pattern: Option<String>) -> Self {
        Self {
            from_date: None,
            to_date: None,
            action_pattern: pattern,
            status_filter: None,
        }
    }

    pub fn with_status(status: Option<bool>) -> Self {
        Self {
            from_date: None,
            to_date: None,
            action_pattern: None,
            status_filter: status,
        }
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
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("audit.") && name.ends_with(".log") && &path != log_path {
                        log_files.push(path);
                    }
                }
            }
        }

        for file_path in &log_files {
            if let Ok(file) = fs::File::open(file_path) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if let Some(entry) = Self::parse_entry(&line) {
                            if self.matches(&entry) {
                                entries.push(entry);
                            }
                        }
                    }
                }
            }
        }

        Ok(entries)
    }

    fn parse_entry(line: &str) -> Option<AuditEntry> {
        let line = line.trim();
        if !line.starts_with('[') {
            return None;
        }
        let close_bracket = line.find(']')?;
        let timestamp = line[1..close_bracket].to_string();

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

    fn matches(&self, entry: &AuditEntry) -> bool {
        if let Some(from) = self.from_date {
            if let Ok(entry_date) =
                NaiveDate::parse_from_str(&entry.timestamp[..10], "%Y-%m-%d")
            {
                if entry_date < from {
                    return false;
                }
            }
        }
        if let Some(to) = self.to_date {
            if let Ok(entry_date) =
                NaiveDate::parse_from_str(&entry.timestamp[..10], "%Y-%m-%d")
            {
                if entry_date > to {
                    return false;
                }
            }
        }
        if let Some(ref pattern) = self.action_pattern {
            if !entry.tool.contains(pattern) {
                return false;
            }
        }
        if let Some(status) = self.status_filter {
            if entry.approved != status {
                return false;
            }
        }
        true
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
