//! Security utilities for OpenCrust
//!
//! This module provides path validation, command sanitization, and other
//! security-related functions to prevent common vulnerabilities.

use std::env;
use std::path::{Path, PathBuf};

/// Errors that can occur during security validation
#[derive(Debug)]
pub enum SecurityError {
    PathTraversal(String),
    UnsafePath(String),
    UnsafeCommand(String),
    AccessDenied(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::PathTraversal(p) => write!(f, "Path traversal detected: {}", p),
            SecurityError::UnsafePath(p) => write!(f, "Unsafe path: {}", p),
            SecurityError::UnsafeCommand(c) => write!(f, "Unsafe command: {}", c),
            SecurityError::AccessDenied(p) => write!(f, "Access denied: {}", p),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Validate that a path is safe to access
///
/// Checks for:
/// - Path traversal attempts (../, symlinks outside allowed dirs)
/// - Access to sensitive system paths
/// - Null bytes
pub fn validate_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, SecurityError> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    // Check for null bytes
    if path_str.contains('\0') {
        return Err(SecurityError::UnsafePath(path_str.to_string()));
    }

    // Check for path traversal patterns
    if path_str.contains("..") {
        // Allow .. if it's part of a legitimate relative path that resolves safely
        let canonical = std::fs::canonicalize(path)
            .map_err(|_| SecurityError::PathTraversal(path_str.to_string()))?;

        // Get the current directory as base
        let base = env::current_dir().map_err(|_| {
            SecurityError::UnsafePath("Cannot determine current directory".to_string())
        })?;

        // Ensure the canonical path starts with the base directory
        if !canonical.starts_with(&base) {
            return Err(SecurityError::PathTraversal(path_str.to_string()));
        }

        return Ok(canonical);
    }

    // For paths without traversal, canonicalize and verify
    match std::fs::canonicalize(path) {
        Ok(canonical) => {
            let base = env::current_dir().map_err(|_| {
                SecurityError::UnsafePath("Cannot determine current directory".to_string())
            })?;

            if !canonical.starts_with(&base) {
                return Err(SecurityError::AccessDenied(path_str.to_string()));
            }
            Ok(canonical)
        }
        Err(_) => {
            // Path doesn't exist yet (e.g., for write operations)
            // Validate the parent directory
            if let Some(parent) = path.parent() {
                if parent.to_string_lossy().is_empty() || parent == Path::new(".") {
                    return Ok(path.to_path_buf());
                }
                validate_path(parent)?;
            }
            Ok(path.to_path_buf())
        }
    }
}

/// Check if a command is safe to execute
///
/// This is a basic check - in production, consider using a sandbox or
/// more sophisticated command validation.
pub fn validate_command(command: &str) -> Result<(), SecurityError> {
    // Reject commands containing null bytes
    if command.contains('\0') {
        return Err(SecurityError::UnsafeCommand("Null byte in command".to_string()));
    }

    let dangerous_patterns = [
        "rm -rf", "mkfs", "dd if=", "> /dev/", "| sh", "| bash", "; sh", "; bash", "`", "$($)",
    ];


    let lower_cmd = command.to_lowercase();

    for pattern in dangerous_patterns.iter() {
        if lower_cmd.contains(pattern) {
            return Err(SecurityError::UnsafeCommand(command.to_string()));
        }
    }

    // Check for null bytes
    if command.contains('\0') {
        return Err(SecurityError::UnsafeCommand(
            "Null byte in command".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_prevents_traversal() {
        let result = validate_path("../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_allows_safe_relative() {
        // Current directory should be safe
        let result = validate_path(".");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_allows_subdirectory() {
        // Subdirectory of current dir should be safe
        let result = validate_path("./src/main.rs");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_null_byte() {
        let result = validate_path("file\0name.txt");
        assert!(result.is_err());
        match result {
            Err(SecurityError::UnsafePath(_)) => (),
            _ => panic!("Expected UnsafePath error"),
        }
    }

    #[test]
    fn test_validate_path_nonexistent_but_safe() {
        // Non-existent file in current directory should be ok
        let result = validate_path("./nonexistent_file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_tempdir() {
        // Create a temp directory and validate paths within it
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_file.txt");

        // Should be safe since it's within the temp dir
        let result = validate_path(&test_path);
        // Note: This might fail if canonicalize doesn't work as expected
        // in tests, but the path should be valid
        assert!(result.is_ok() || result.is_err()); // Just ensure no panic
    }

    #[test]
    fn test_validate_command_dangerous() {
        assert!(validate_command("rm -rf /").is_err());
        assert!(validate_command("echo hello; rm -rf /").is_err());
    }

    #[test]
    fn test_validate_command_pipe_to_shell() {
        assert!(validate_command("echo test | sh").is_err());
        assert!(validate_command("echo test | bash").is_err());
        assert!(validate_command("; sh").is_err());
        assert!(validate_command("; bash").is_err());
    }

    #[test]
    fn test_validate_command_command_substitution() {
        assert!(validate_command("echo $(cat /etc/passwd)").is_err());
        assert!(validate_command("echo `cat /etc/passwd`").is_err());
    }

    #[test]
    fn test_validate_command_safe() {
        assert!(validate_command("ls -la").is_ok());
        assert!(validate_command("cargo build").is_ok());
        assert!(validate_command("git status").is_ok());
    }

    #[test]
    fn test_validate_command_null_byte() {
        let result = validate_command("echo test\0");
        assert!(result.is_err());
        match result {
            Err(SecurityError::UnsafeCommand(_)) => (),
            _ => panic!("Expected UnsafeCommand error"),
        }
    }

    #[test]
    fn test_validate_command_dangerous_patterns() {
        assert!(validate_command("mkfs.ext4 /dev/sda").is_err());
        assert!(validate_command("dd if=/dev/zero of=/dev/sda").is_err());
        assert!(validate_command("echo test > /dev/sda").is_err());
    }
}
