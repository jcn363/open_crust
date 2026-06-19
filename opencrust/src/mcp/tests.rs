use super::*;

#[test]
fn test_validate_mcp_command_allowed() {
    assert!(validate_mcp_command("npx").is_ok());
    assert!(validate_mcp_command("node").is_ok());
    assert!(validate_mcp_command("python").is_ok());
    assert!(validate_mcp_command("python3").is_ok());
    assert!(validate_mcp_command("pip").is_ok());
    assert!(validate_mcp_command("pip3").is_ok());
    assert!(validate_mcp_command("cargo").is_ok());
    assert!(validate_mcp_command("uvx").is_ok());
    assert!(validate_mcp_command("uv").is_ok());
    assert!(validate_mcp_command("go").is_ok());
    assert!(validate_mcp_command("java").is_ok());
    assert!(validate_mcp_command("mcp-server").is_ok());
    // Test with full path
    assert!(validate_mcp_command("/usr/bin/npx").is_ok());
    assert!(validate_mcp_command("/usr/local/bin/node").is_ok());
}

#[test]
fn test_validate_mcp_command_rejected() {
    assert!(validate_mcp_command("ls").is_err());
    assert!(validate_mcp_command("rm").is_err());
    assert!(validate_mcp_command("sh").is_err());
    assert!(validate_mcp_command("bash").is_err());
    assert!(validate_mcp_command("evil-command").is_err());
    assert!(validate_mcp_command("/tmp/malicious").is_err());
}
