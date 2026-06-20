use crate::cli::McpCommands;
use crate::config::Config;
use crate::error::{OpenCrustError, Result};
use crate::mcp::McpManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_mcp(
    cmd: McpCommands,
    config: &Config,
    __mcp_manager: Arc<Mutex<McpManager>>,
) -> Result<()> {
    match cmd {
        McpCommands::List => {
            println!("Available MCP servers (curated list):");
            println!("\n=== Tier 1: Essential ===");
            println!(
                "  context7         - Version-accurate library docs (eliminates API hallucinations)"
            );
            println!("  github          - GitHub integration (repos, issues, PRs, CI/CD)");
            println!("  postgres        - PostgreSQL database queries");
            println!("  brave-search    - Web search (privacy-focused)");
            println!("  filesystem      - Enhanced file system access");
            println!("  sequentialthinking - Structured thinking and reasoning");
            println!("\n=== Tier 2: High Value ===");
            println!("  playwright      - Browser automation & E2E testing");
            println!("  supabase        - RLS-aware database access");
            println!("  sentry          - Error monitoring integration");
            println!("  linear          - Issue tracking");
            println!("  e2b             - Secure cloud sandbox for code execution");
            println!("  octocode        - Code analysis and refactoring");
            println!("\n=== Tier 3: Production ===");
            println!("  slack           - Slack messaging");
            println!("  google-drive    - Google Drive file access");
            println!("  stripe          - Payment integration (requires OAuth)");
            println!("\nUse `opencrust mcp install <name>` to add a server.");
            println!("For more servers, visit: https://github.com/modelcontextprotocol/servers");
            println!("Or browse: https://mcpdirectory.app/ (2,500+ servers)");
        }
        McpCommands::Install { server } => {
            let mut new_config = config.clone();
            let (command, description, env_help) = match server.as_str() {
                // Tier 1: Essential
                "context7" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@context7/mcp-server".to_string(),
                    ],
                    "Version-accurate library docs".to_string(),
                    "No API key required",
                ),
                "github" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-github".to_string(),
                    ],
                    "GitHub integration (repos, issues, PRs)".to_string(),
                    "Set GITHUB_TOKEN env var",
                ),
                "postgres" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-postgres".to_string(),
                    ],
                    "PostgreSQL database queries".to_string(),
                    "Set DATABASE_URL env var",
                ),
                "brave-search" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-brave-search".to_string(),
                    ],
                    "Web search (privacy-focused)".to_string(),
                    "Set BRAVE_API_KEY env var",
                ),
                "filesystem" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                    ],
                    "Enhanced file system access".to_string(),
                    "Set ALLOWED_DIRS env var",
                ),
                "sequentialthinking" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-sequential-thinking".to_string(),
                    ],
                    "Structured thinking and reasoning".to_string(),
                    "No API key required",
                ),
                // Tier 2: High Value
                "playwright" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-playwright".to_string(),
                    ],
                    "Browser automation & E2E testing".to_string(),
                    "Run: npx playwright install",
                ),
                "supabase" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@supabase/mcp-server-supabase".to_string(),
                    ],
                    "RLS-aware database access".to_string(),
                    "Set SUPABASE_ACCESS_TOKEN env var",
                ),
                "sentry" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-sentry".to_string(),
                    ],
                    "Error monitoring integration".to_string(),
                    "Set SENTRY_AUTH_TOKEN env var",
                ),
                "linear" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-linear".to_string(),
                    ],
                    "Issue tracking".to_string(),
                    "Set LINEAR_API_KEY env var",
                ),
                "e2b" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@e2b/mcp-server".to_string(),
                    ],
                    "Secure cloud sandbox for code execution".to_string(),
                    "Set E2B_API_KEY env var",
                ),
                "octocode" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@octocode/mcp-server".to_string(),
                    ],
                    "Code analysis and refactoring".to_string(),
                    "No API key required",
                ),
                // Tier 3: Production
                "slack" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-slack".to_string(),
                    ],
                    "Slack messaging".to_string(),
                    "Set SLACK_TOKEN env var",
                ),
                "google-drive" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-google-drive".to_string(),
                    ],
                    "Google Drive file access".to_string(),
                    "OAuth required",
                ),
                "stripe" => (
                    vec![
                        "npx".to_string(),
                        "-y".to_string(),
                        "@modelcontextprotocol/server-stripe".to_string(),
                    ],
                    "Payment integration".to_string(),
                    "Set STRIPE_API_KEY env var",
                ),
                _ => {
                    eprintln!(
                        "Unknown MCP server: {}. Use `opencrust mcp list` to see available servers.",
                        server
                    );
                    return Ok(());
                }
            };
            let mcp_config = crate::config::McpConfig {
                command,
                environment: None,
                enabled: true,
            };
            new_config.mcp.insert(server.clone(), mcp_config);
            new_config.save();
            println!("Installed MCP server '{}'.", server);
            println!("  Description: {}", description);
            println!("  Setup: {}", env_help);
            println!("\nRestart opencrust to use the server.");
        }
        McpCommands::Browse => {
            println!("MCP Showcase TUI Browser");
            println!();
            println!("To launch the MCP Showcase TUI:");
            println!("  1. Run 'opencrust' without arguments to start the interactive TUI");
            println!("  2. Press Ctrl+M to open the MCP Showcase server browser");
            println!("  3. Navigate with arrow keys, toggle servers with Enter");
            println!("  4. Press Esc to return to the main chat");
            println!();
            println!("CLI alternatives:");
            println!("  'opencrust mcp showcase'  - Print server table to terminal");
            println!("  'opencrust mcp tools'     - List all tools");
            println!("  'opencrust mcp test <server> <tool> [args]' - Execute a tool");
        }
        McpCommands::Showcase => {
            let config = Config::load();
            println!("=== MCP Showcase ===");
            println!();
            if config.mcp.is_empty() {
                println!("No MCP servers configured.");
                println!("Use 'opencrust mcp list' to see available servers.");
                println!("Use 'opencrust mcp install <name>' to install a server.");
            } else {
                println!("{:<20} {:<15} {:<50}", "Name", "Status", "Command");
                println!("{:<20} {:<15} {:<50}", "----", "------", "-------");
                for (name, mcp_config) in &config.mcp {
                    let status = if mcp_config.enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    };
                    let cmd = mcp_config.command.join(" ");
                    let cmd_display = if cmd.len() > 47 {
                        format!("{}...", &cmd[..47])
                    } else {
                        cmd
                    };
                    println!("{:<20} {:<15} {:<50}", name, status, cmd_display);
                }
            }
        }
        McpCommands::Test { server, tool, args } => {
            let config = Config::load();
            let mcp_manager = Arc::new(Mutex::new(crate::mcp::McpManager::new()));
            mcp_manager.lock().await.load_from_config(&config.mcp).await;

            let arguments = match args {
                Some(json_str) => {
                    serde_json::from_str(json_str.as_str()).map_err(|e| OpenCrustError::Json(e))
                }
                None => Ok(serde_json::json!({})),
            };

            match arguments {
                Ok(args_val) => {
                    let full_name = format!("{}_{}", server, tool);
                    println!("Calling MCP tool '{}' on server '{}'...", tool, server);
                    println!(
                        "Arguments: {}",
                        serde_json::to_string_pretty(&args_val).unwrap_or_default()
                    );
                    println!();
                    match mcp_manager
                        .lock()
                        .await
                        .call_tool(&full_name, &args_val)
                        .await
                    {
                        Ok(result) => {
                            println!("=== Result ===");
                            println!("{}", result);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        McpCommands::Tools => {
            let config = Config::load();
            let mcp_manager = Arc::new(Mutex::new(crate::mcp::McpManager::new()));
            mcp_manager.lock().await.load_from_config(&config.mcp).await;

            println!("=== MCP Tools ===");
            println!();

            let tools = mcp_manager.lock().await.list_tools().await;
            if tools.is_empty() {
                println!("No tools found. Make sure you have MCP servers configured and enabled.");
                println!("Use 'opencrust mcp list' to see available servers.");
                println!("Use 'opencrust mcp install <name>' to install a server.");
            } else {
                println!("Found {} tools:", tools.len());
                println!();
                for tool in &tools {
                    let name = tool
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let desc = tool
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No description");
                    println!("  {} {}", name, desc);
                    if let Some(schema) = tool.get("inputSchema")
                        && let Some(props) = schema.get("properties")
                        && let Some(props_obj) = props.as_object()
                        && !props_obj.is_empty()
                    {
                        println!("    Arguments:");
                        for (prop_name, prop_info) in props_obj {
                            let prop_type = prop_info
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("any");
                            let prop_desc = prop_info
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if prop_desc.is_empty() {
                                println!("      - {}: {}", prop_name, prop_type);
                            } else {
                                println!("      - {} ({}): {}", prop_name, prop_type, prop_desc);
                            }
                        }
                    }
                    println!();
                }
            }
        }
    }
    Ok(())
}
