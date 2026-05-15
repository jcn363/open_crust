
//! MCP Showcase module for browsing, installing, and managing MCP servers
//! Provides TUI (ratatui) interface;

pub mod tui;
pub use tui::McpShowcaseAction;
pub use tui::McpShowcaseUI;

/// Public information about an MCP server
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub enabled: bool,
}

/// Get a human-readable description for known MCP servers
pub fn get_server_description(server_name: &str) -> String {
    match server_name {
        "context7" => "Version-accurate library docs (eliminates API hallucinations)".to_string(),
        "github" => "Repository management, issues, PRs, CI/CD".to_string(),
        "brave-search" => "Privacy-focused web search".to_string(),
        "postgres" => "Natural language DB queries".to_string(),
        "filesystem" => "Enhanced file operations".to_string(),
        "playwright" => "Browser automation & E2E testing".to_string(),
        "supabase" => "RLS-aware database access".to_string(),
        "sentry" => "Error monitoring".to_string(),
        "linear" => "Issue tracking".to_string(),
        "e2b" => "Secure cloud sandbox for code execution".to_string(),
        "mcpdirectory" => "Central registry of MCP servers".to_string(),
        _ => "Unknown server".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_server_description() {
        assert_eq!(
            get_server_description("context7"),
            "Version-accurate library docs (eliminates API hallucinations)"
        );
        assert_eq!(
            get_server_description("github"),
            "Repository management, issues, PRs, CI/CD"
        );
        assert_eq!(get_server_description("unknown-server"), "Unknown server");
    }

    #[test]
    fn test_module_structure() {
        // Just verify the module compiles and exports the expected items
        let _ = get_server_description("test");
        let _ = tui::McpShowcaseUI::new(vec![]);
    }
}
