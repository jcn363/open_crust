use super::*;

#[test]
fn test_supported_extensions_not_empty() {
    let exts = supported_extensions();
    assert!(!exts.is_empty());
    assert!(exts.contains(&"rs"));
    assert!(exts.contains(&"py"));
    assert!(exts.contains(&"js"));
}

#[test]
fn test_has_formatter_known_extensions() {
    assert!(has_formatter("rs"));
    assert!(has_formatter("py"));
    assert!(has_formatter("js"));
    assert!(has_formatter("ts"));
    assert!(has_formatter("json"));
}

#[test]
fn test_has_formatter_unknown_extension() {
    assert!(!has_formatter("xyz"));
    assert!(!has_formatter("dat"));
}

#[test]
fn test_format_file_no_formatter_for_extension() {
    let result = format_file(Path::new("test.xyz"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("No formatter configured"));
}

#[test]
fn test_format_file_nonexistent_rust_file() {
    // Should fail gracefully when file doesn't exist
    let result = format_file(Path::new("/nonexistent/test.rs"));
    assert!(result.is_err());
}
