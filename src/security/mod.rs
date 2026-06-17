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
    PromptInjection(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::PathTraversal(p) => write!(f, "Path traversal detected: {}", p),
            SecurityError::UnsafePath(p) => write!(f, "Unsafe path: {}", p),
            SecurityError::UnsafeCommand(c) => write!(f, "Unsafe command: {}", c),
            SecurityError::AccessDenied(p) => write!(f, "Access denied: {}", p),
            SecurityError::PromptInjection(p) => write!(f, "Prompt injection detected: {}", p),
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

/// Check if a prompt contains potential injection attempts.
///
/// Detects common prompt injection patterns:
/// - System prompt override attempts ("ignore previous instructions", "you are now...")
/// - Role manipulation ("act as", "pretend to be")
/// - Delimiter injection (XML/HTML tags, markdown code blocks)
/// - Instruction leakage attempts
///
/// Returns Ok(()) if safe, Err(PromptInjection) if suspicious patterns detected.
pub fn check_prompt_injection(prompt: &str) -> Result<(), SecurityError> {
    let lower = prompt.to_lowercase();

    // System prompt override attempts
    let override_patterns = [
        "ignore previous",
        "ignore all previous",
        "disregard previous",
        "forget previous",
        "override previous",
        "new instructions:",
        "system prompt:",
        "you are now",
        "you will now",
        "from now on you",
        "act as if",
        "pretend you are",
        "simulate being",
        "roleplay as",
    ];

    for pattern in &override_patterns {
        if lower.contains(pattern) {
            return Err(SecurityError::PromptInjection(format!(
                "Contains override pattern: '{}'",
                pattern
            )));
        }
    }

    // Delimiter injection (XML/HTML tags that might confuse the parser)
    let delimiter_patterns = ["<system>", "</system>", "<instruction>", "</instruction>"];

    for pattern in &delimiter_patterns {
        if lower.contains(pattern) {
            return Err(SecurityError::PromptInjection(format!(
                "Contains delimiter injection: '{}'",
                pattern
            )));
        }
    }

    // Instruction leakage attempts
    let leakage_patterns = [
        "repeat your instructions",
        "show me your prompt",
        "what are your instructions",
        "print your system prompt",
        "output your instructions",
        "reveal your instructions",
    ];

    for pattern in &leakage_patterns {
        if lower.contains(pattern) {
            return Err(SecurityError::PromptInjection(format!(
                "Contains instruction leakage attempt: '{}'",
                pattern
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
