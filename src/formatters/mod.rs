//! Auto-formatter integration
//!
//! Detects file type by extension and runs the appropriate formatter
//! (`cargo fmt` for Rust, `prettier` for JS/TS, etc.) on save or on demand.
//! Supports configurable formatters per language via `FormatterConfig`.

use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Output};

/// Configuration for a single formatter command
#[derive(Debug, Clone)]
pub struct FormatterCommand {
    /// The command to run (e.g., "prettier", "black")
    pub command: String,
    /// Arguments to pass before the file path (e.g., ["--write"])
    pub args: Vec<String>,
}

impl FormatterCommand {
    pub fn new(command: &str, args: &[&str]) -> Self {
        Self {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Default formatter mappings by file extension
fn default_formatters() -> HashMap<&'static str, FormatterCommand> {
    let mut map = HashMap::new();

    // Rust
    map.insert(
        "rs",
        FormatterCommand::new("rustfmt", &["--edition", "2024"]),
    );

    // JavaScript/TypeScript/JSON/Markdown via Prettier
    map.insert("js", FormatterCommand::new("prettier", &["--write"]));
    map.insert("jsx", FormatterCommand::new("prettier", &["--write"]));
    map.insert("ts", FormatterCommand::new("prettier", &["--write"]));
    map.insert("tsx", FormatterCommand::new("prettier", &["--write"]));
    map.insert("json", FormatterCommand::new("prettier", &["--write"]));
    map.insert("md", FormatterCommand::new("prettier", &["--write"]));
    map.insert("css", FormatterCommand::new("prettier", &["--write"]));
    map.insert("scss", FormatterCommand::new("prettier", &["--write"]));
    map.insert("html", FormatterCommand::new("prettier", &["--write"]));
    map.insert("yaml", FormatterCommand::new("prettier", &["--write"]));
    map.insert("yml", FormatterCommand::new("prettier", &["--write"]));

    // Python
    map.insert("py", FormatterCommand::new("black", &["--quiet"]));

    // Go
    map.insert("go", FormatterCommand::new("gofmt", &["-w"]));

    // Ruby
    map.insert("rb", FormatterCommand::new("rubocop", &["-a", "-q"]));

    // PHP
    map.insert("php", FormatterCommand::new("php-cs-fixer", &["fix"]));

    // Swift
    map.insert("swift", FormatterCommand::new("swift-format", &["-i"]));

    // C/C++
    map.insert("c", FormatterCommand::new("clang-format", &["-i"]));
    map.insert("cpp", FormatterCommand::new("clang-format", &["-i"]));
    map.insert("h", FormatterCommand::new("clang-format", &["-i"]));
    map.insert("hpp", FormatterCommand::new("clang-format", &["-i"]));

    // Java
    map.insert("java", FormatterCommand::new("google-java-format", &["-i"]));

    map
}

/// Format a file in-place using the appropriate formatter for its extension.
/// Returns `Ok(())` on success, or an error message string on failure.
pub fn format_file(path: &Path) -> Result<(), String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let formatters = default_formatters();
    let formatter = formatters
        .get(extension.as_str())
        .ok_or_else(|| format!("No formatter configured for .{} files", extension))?;

    let mut cmd = Command::new(&formatter.command);
    for arg in &formatter.args {
        cmd.arg(arg);
    }
    cmd.arg(path);

    match cmd.output() {
        Ok(Output { status, stderr, .. }) if !status.success() => {
            let msg = String::from_utf8_lossy(&stderr);
            Err(format!(
                "Formatter '{}' failed for {}: {}",
                formatter.command,
                path.display(),
                msg.trim()
            ))
        }
        Err(e) => Err(format!(
            "Could not run formatter '{}' for {}: {}",
            formatter.command,
            path.display(),
            e
        )),
        Ok(_) => Ok(()),
    }
}

/// Get the list of supported file extensions
#[cfg(test)]
pub fn supported_extensions() -> Vec<&'static str> {
    let mut exts: Vec<_> = default_formatters().into_keys().collect();
    exts.sort_by_key(|e| e.to_lowercase());
    exts
}

/// Check if a file extension has a configured formatter
#[cfg(test)]
pub fn has_formatter(extension: &str) -> bool {
    default_formatters().contains_key(extension)
}

#[cfg(test)]
mod tests;
