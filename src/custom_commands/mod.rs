//! Custom commands — user-extendable slash command system
//!
//! Discovers executable scripts from `.opencrust/commands/`.
//! Each script is auto-discovered at startup and registered as a slash command.
//! Parses script headers for name, description, and optional keybind metadata.
//!
//! Script header format:
//! ```bash
//! # name: my-command
//! # description: Does something useful
//! # keybind: Ctrl+Shift+M
//! ```
//!
//! The script receives the command arguments as positional parameters.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A custom command discovered from `.opencrust/commands/`
pub struct CustomCommand {
    /// Command name (used as `/name` in the input)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Optional keybind (e.g., "Ctrl+Shift+M")
    #[allow(dead_code)] // wired for future keybind handling
    pub keybind: Option<String>,
    /// Path to the executable script
    pub path: PathBuf,
}

/// Manager for custom commands
pub struct CustomCommandManager {
    pub commands: HashMap<String, CustomCommand>,
}

impl CustomCommandManager {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Discover custom commands from `.opencrust/commands/` directories.
    pub fn discover(&mut self) {
        let paths = vec![
            PathBuf::from(".opencrust/commands"),
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("opencrust/commands"),
        ];

        for path in paths {
            if path.exists()
                && path.is_dir()
                && let Ok(entries) = fs::read_dir(path)
            {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file()
                        && let Some(cmd) = self.parse_command(&p)
                    {
                        self.commands.insert(cmd.name.clone(), cmd);
                    }
                }
            }
        }
    }

    /// Parse a command script's header for metadata.
    fn parse_command(&self, path: &Path) -> Option<CustomCommand> {
        let content = fs::read_to_string(path).ok()?;
        let mut name = None;
        let mut description = String::new();
        let mut keybind = None;

        for line in content.lines() {
            if line.starts_with("# name:") {
                name = Some(line.trim_start_matches("# name:").trim().to_string());
            } else if line.starts_with("# description:") {
                description = line.trim_start_matches("# description:").trim().to_string();
            } else if line.starts_with("# keybind:") {
                keybind = Some(line.trim_start_matches("# keybind:").trim().to_string());
            }
        }

        let name = name.or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })?;

        Some(CustomCommand {
            name,
            description,
            keybind,
            path: path.to_path_buf(),
        })
    }

    /// Execute a custom command with the given arguments string.
    /// Returns the command output or an error message.
    pub fn execute_command(&self, name: &str, args: &str) -> Result<String, String> {
        let cmd = self
            .commands
            .get(name)
            .ok_or_else(|| format!("Command '/{}' not found", name))?;

        // Split args by whitespace and pass as positional arguments
        let args_vec: Vec<&str> = args.split_whitespace().collect();

        let output = Command::new(&cmd.path)
            .args(&args_vec)
            .output()
            .map_err(|e| format!("Failed to execute command '{}': {}", name, e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("Command '{}' failed with exit code {}", name, output.status)
            } else {
                format!("Command '{}' failed: {}", name, stderr)
            })
        }
    }

    /// Get a list of all registered custom command names and descriptions.
    #[allow(dead_code)] // used by tests; dead in binary cfg
    pub fn list_commands(&self) -> Vec<(String, String)> {
        self.commands
            .values()
            .map(|cmd| (cmd.name.clone(), cmd.description.clone()))
            .collect()
    }

    /// Check if a command name exists
    pub fn has_command(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
}

#[cfg(test)]
mod tests;
