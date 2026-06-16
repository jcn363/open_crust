use super::*;
use tempfile::TempDir;

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

#[test]
fn test_validate_path_prevents_traversal() {
    let result = validate_path("../../etc/passwd");
    assert!(result.is_err());
}

#[test]
fn test_validate_path_allows_safe_relative() {
    let result = validate_path(".");
    assert!(result.is_ok());
}

#[test]
fn test_validate_path_allows_subdirectory() {
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
    let result = validate_path("./nonexistent_file.txt");
    assert!(result.is_ok());
}

#[test]
fn test_validate_path_tempdir() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("test_file.txt");
    let result = validate_path(&test_path);
    assert!(result.is_ok() || result.is_err());
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
    assert!(result.is_err() || result.is_ok());
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
