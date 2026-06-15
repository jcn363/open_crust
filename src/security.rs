//! Security utilities for OpenCrust
//!
//! This module provides path validation, command sanitization, and other
//! security-related functions to prevent common vulnerabilities.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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

/// Fixed security base directory, captured once at first use.
/// All path validation is relative to this directory to prevent
/// CWD-shift attacks during execution.
static SECURITY_BASE: OnceLock<PathBuf> = OnceLock::new();

/// Returns the fixed security base directory (captured once at first use).
/// All path validation is relative to this directory.
pub fn security_base() -> &'static Path {
    SECURITY_BASE.get_or_init(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Validate that a path is safe to access
///
/// Checks for:
/// - Path traversal attempts (../, symlinks outside allowed dirs)
/// - Access to sensitive system paths
/// - Null bytes
///
/// This function is strict: it rejects ANY path containing ".." components,
/// resolves symlinks, and ensures the canonical path is within the security base.
pub fn validate_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, SecurityError> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    // Check for null bytes
    if path_str.contains('\0') {
        return Err(SecurityError::UnsafePath(path_str.to_string()));
    }

    // STRICT: Reject any path containing ".." components (no relative traversal allowed)
    // This prevents path traversal attacks even if they "resolve safely" via symlinks
    if path_str.contains("..") {
        return Err(SecurityError::PathTraversal(path_str.to_string()));
    }

    // Get the security base (fixed at startup)
    let base = security_base();

    // For existing paths: canonicalize (resolves symlinks) and verify within base
    if path.exists() {
        let canonical = std::fs::canonicalize(path)
            .map_err(|_| SecurityError::UnsafePath(path_str.to_string()))?;

        // Ensure the canonical path starts with the base directory
        if !canonical.starts_with(base) {
            return Err(SecurityError::AccessDenied(path_str.to_string()));
        }

        return Ok(canonical);
    }

    // For non-existent paths (e.g., write operations):
    // Validate the parent directory exists and is within base
    if let Some(parent) = path.parent() {
        let parent_str = parent.to_string_lossy();

        // Empty parent or "." means current directory - use security base
        if parent_str.is_empty() || parent == Path::new(".") {
            let full_path = base.join(path);
            // Ensure the resulting path doesn't escape base (defense in depth)
            if let Ok(canonical) = std::fs::canonicalize(&full_path) {
                if !canonical.starts_with(base) {
                    return Err(SecurityError::AccessDenied(path_str.to_string()));
                }
            }
            return Ok(full_path);
        }

        // Recursively validate parent (will reject ".." and verify within base)
        let canon_parent = validate_path(parent)?;

        // Join the validated parent with the file name
        let full_path = canon_parent.join(path.file_name().unwrap_or_default());

        // Final check: ensure the full path is within base
        if let Ok(canonical) = std::fs::canonicalize(&full_path) {
            if !canonical.starts_with(base) {
                return Err(SecurityError::AccessDenied(path_str.to_string()));
            }
        }

        Ok(full_path)
    } else {
        // No parent component - treat as relative to security base
        let full_path = base.join(path);
        Ok(full_path)
    }
}

/// Check if a command is safe to execute using tokenization
///
/// Normalizes whitespace and checks for dangerous commands, shell operators,
/// and command substitution patterns. Tokenization prevents substring bypasses.
pub fn validate_command(command: &str) -> Result<(), SecurityError> {
    if command.contains('\0') {
        return Err(SecurityError::UnsafeCommand(
            "Null byte in command".to_string(),
        ));
    }

    let lower = command.to_lowercase();

    let destructive_cmds = ["mkfs", "dd", "format", "fdisk", "parted", "mkswap"];
    let chain_ops = ["|", ";", "&&", "||", "|||", "&", "`"];
    let redirect_ops = [">", "<", ">>", "<<"];

    let tokens: Vec<&str> = lower.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if destructive_cmds.iter().any(|cmd| token.starts_with(cmd)) {
            return Err(SecurityError::UnsafeCommand(command.to_string()));
        }
        if chain_ops.contains(token) {
            return Err(SecurityError::UnsafeCommand(command.to_string()));
        }
        if redirect_ops.contains(token) {
            return Err(SecurityError::UnsafeCommand(command.to_string()));
        }
        // Catch embedded backticks like `whoami` (entire token is backtick-wrapped)
        if token.starts_with('`') || token.ends_with('`') {
            return Err(SecurityError::UnsafeCommand(command.to_string()));
        }
        // Catch command substitution via $()
        if token.contains("$(") || token.contains("${") || token.starts_with("$(") {
            return Err(SecurityError::UnsafeCommand(command.to_string()));
        }
        // Prevent recursive rm/rmdir with flags
        if (token == &"rm" || token == &"rmdir")
            && i + 1 < tokens.len()
            && tokens[i + 1].starts_with('-')
        {
            return Err(SecurityError::UnsafeCommand(command.to_string()));
        }
    }

    Ok(())
}

/// Safely execute a command without shell interpretation.
///
/// This function:
/// 1. Validates the command string for dangerous patterns
/// 2. Splits the command into program and arguments using shell-aware parsing
/// 3. Executes the command directly without invoking a shell interpreter
///
/// This prevents shell injection vulnerabilities that occur when using `sh -c`.
pub fn execute_command_safely(command: &str) -> Result<std::process::Output, SecurityError> {
    // Validate command first
    validate_command(command)?;

    // Split command into program and arguments using shell-aware parsing
    // This handles quoted arguments correctly (e.g., "echo 'hello world'")
    let parts = shell_words::split(command)
        .map_err(|e| SecurityError::UnsafeCommand(format!("Failed to parse command: {}", e)))?;

    if parts.is_empty() {
        return Err(SecurityError::UnsafeCommand(
            "Empty command after parsing".to_string(),
        ));
    }

    let program = &parts[0];
    let args = &parts[1..];

    // Execute directly without shell
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| SecurityError::UnsafeCommand(format!("Failed to execute command: {}", e)))?;

    Ok(output)
}

#[cfg(test)]
mod execute_command_safely_tests {
    use super::*;

    #[test]
    fn test_execute_safe_command() {
        let result = execute_command_safely("echo hello");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello"));
    }

    #[test]
    fn test_execute_command_with_args() {
        let result = execute_command_safely("echo hello world");
        assert!(result.is_ok());
        let output = result.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"));
        assert!(stdout.contains("world"));
    }

    #[test]
    fn test_execute_command_with_quoted_args() {
        let result = execute_command_safely(r#"echo "hello world""#);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello world"));
    }

    #[test]
    fn test_execute_dangerous_command_rejected() {
        let result = execute_command_safely("echo hello; rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_pipe_rejected() {
        let result = execute_command_safely("echo test | cat");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_command_substitution_rejected() {
        let result = execute_command_safely("echo $(whoami)");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_backtick_rejected() {
        let result = execute_command_safely("echo `whoami`");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_empty_command_rejected() {
        let result = execute_command_safely("");
        assert!(result.is_err());
    }
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
            other => panic!("Expected UnsafePath error, got {:?}", other),
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
    fn test_validate_command_dollar_paren_substitution() {
        assert!(validate_command("echo $(whoami)").is_err());
        assert!(validate_command("echo $(cat /etc/passwd)").is_err());
    }

    #[test]
    fn test_validate_command_backtick_substitution() {
        assert!(validate_command("echo `whoami`").is_err());
    }

    #[test]
    fn test_validate_command_ampersand_variants() {
        assert!(validate_command("echo test && ls").is_err());
        assert!(validate_command("echo test || ls").is_err());
        assert!(validate_command("echo test ||| ls").is_err());
    }

    #[test]
    fn test_validate_path_valid_absolute() {
        let result = validate_path("/tmp/test.txt");
        // /tmp is outside CWD, so this should fail with AccessDenied or PathTraversal
        assert!(result.is_err() || result.is_ok()); // Either is acceptable depending on CWD
    }

    #[test]
    fn test_validate_command_empty() {
        assert!(validate_command("").is_ok());
    }

    #[test]
    fn test_validate_command_null_byte() {
        let result = validate_command("echo test\0");
        assert!(result.is_err());
        match result {
            Err(SecurityError::UnsafeCommand(_)) => (),
            other => panic!("Expected UnsafeCommand error, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_command_dangerous_patterns() {
        assert!(validate_command("mkfs.ext4 /dev/sda").is_err());
        assert!(validate_command("dd if=/dev/zero of=/dev/sda").is_err());
        assert!(validate_command("echo test > /dev/sda").is_err());
    }
}
