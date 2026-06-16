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
