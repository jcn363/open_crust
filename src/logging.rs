//! Logging utilities for OpenCrust
//!
//! Provides structured logging for provider fallback events and other
//! operational events. Logs are written to a dedicated log file in the
//! config directory.

use chrono::Utc;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Get the log directory path
fn log_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencrust/logs")
}

/// Get the fallback log file path
fn fallback_log_path() -> PathBuf {
    log_dir().join("fallback.log")
}

/// Ensure the log directory exists
fn ensure_log_dir() {
    let dir = log_dir();
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
}

/// Log a provider fallback event
///
/// Records when a fallback provider is successfully used after the primary
/// provider fails. Includes timestamp, primary provider (if known), and
/// fallback provider name.
pub fn log_fallback(fallback_provider: &str) {
    ensure_log_dir();
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let log_entry = format!(
        "[{}] FALLBACK_USED provider={}\n",
        timestamp, fallback_provider
    );

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(fallback_log_path())
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

/// Log a fallback attempt (primary provider failed, fallback will be tried)
pub fn log_fallback_attempt(error_msg: &str) {
    ensure_log_dir();
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    // Extract the primary error from the message
    let error = error_msg
        .strip_prefix("opencrust: Primary provider failed: ")
        .unwrap_or(error_msg)
        .split(". Trying fallback...")
        .next()
        .unwrap_or(error_msg);
    let log_entry = format!(
        "[{}] FALLBACK_ATTEMPT error={}\n",
        timestamp, error
    );

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(fallback_log_path())
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

/// Log a fallback exhaustion event (all fallbacks failed)
#[allow(dead_code, reason = "Used for logging fallback exhaustion events")]
pub fn log_fallback_exhausted(providers_tried: &[String]) {
    ensure_log_dir();
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let log_entry = format!(
        "[{}] FALLBACK_EXHAUSTED providers_tried={}\n",
        timestamp,
        providers_tried.join(",")
    );

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(fallback_log_path())
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

/// Read recent fallback log entries
#[allow(dead_code, reason = "Used for reading fallback log entries")]
pub fn read_fallback_log(limit: usize) -> Result<Vec<String>, std::io::Error> {
    let path = fallback_log_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    Ok(lines.into_iter().rev().take(limit).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_log_fallback_creates_file() {
        let test_provider = "test_provider_openai";
        log_fallback(test_provider);

        let path = fallback_log_path();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("FALLBACK_USED"));
        assert!(content.contains(test_provider));

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_log_fallback_attempt_creates_file() {
        let test_error = "opencrust: Primary provider failed: connection timeout. Trying fallback...";
        log_fallback_attempt(test_error);

        let path = fallback_log_path();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("FALLBACK_ATTEMPT"));
        assert!(content.contains("connection timeout"));

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_log_fallback_exhausted_creates_file() {
        let providers = vec!["openai".to_string(), "anthropic".to_string()];
        log_fallback_exhausted(&providers);

        let path = fallback_log_path();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("FALLBACK_EXHAUSTED"));
        assert!(content.contains("openai"));
        assert!(content.contains("anthropic"));

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_read_fallback_log() {
        log_fallback("provider1");
        log_fallback("provider2");

        let entries = read_fallback_log(10).expect("Failed to read fallback log");
        assert_eq!(entries.len(), 2);
        assert!(entries[0].contains("provider2")); // Most recent first
        assert!(entries[1].contains("provider1"));

        // Cleanup
        let _ = fs::remove_file(fallback_log_path());
    }
}