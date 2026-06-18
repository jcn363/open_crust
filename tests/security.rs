//! Integration tests for security path validation and permission checks.

use opencrust::security;

/// Paths within allowed directories validate correctly.
#[test]
fn allowed_paths_validate() {
    let home = std::env::current_dir().expect("current dir");
    assert!(
        security::validate_path(&home).is_ok(),
        "home dir should be allowed"
    );
}

/// Relative paths that try to escape are rejected.
#[test]
fn path_traversal_is_rejected() {
    let path = std::path::PathBuf::from("../etc/passwd");
    let result = security::validate_path(&path);
    assert!(result.is_err(), "traversal path should be rejected");
}

/// Paths with null bytes are rejected.
#[test]
fn path_with_null_byte_is_rejected() {
    let path = std::path::PathBuf::from("hello\x00world");
    let result = security::validate_path(&path);
    assert!(result.is_err(), "paths with null bytes should be rejected");
}
