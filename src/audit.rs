use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    pub fn new() -> Self {
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("open_crust/logs");

        if !log_dir.exists() {
            let _ = std::fs::create_dir_all(&log_dir);
        }

        Self {
            log_path: log_dir.join("audit.log"),
        }
    }

    pub fn log_action(&self, tool_name: &str, input: &str, approved: bool) {
        let timestamp = Utc::now().to_rfc3339();
        let status = if approved { "APPROVED" } else { "DENIED" };
        let log_entry = format!(
            "[{}] {}: tool={} input={}\n",
            timestamp, status, tool_name, input
        );

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }
}
