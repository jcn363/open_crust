//! Auto-formatter integration
//!
//! Detects file type by extension and runs the appropriate formatter
//! (`cargo fmt` for Rust, `prettier` for JS/TS, etc.) on save.

use std::path::Path;
use std::process::{Command, Output};

pub fn format_file(path: &Path) {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let result = match extension {
        "rs" => Command::new("rustfmt").arg(path).output(),
        "js" | "ts" | "json" | "md" => Command::new("prettier").arg("--write").arg(path).output(),
        _ => return,
    };

    match result {
        Ok(Output { status, stderr, .. }) if !status.success() => {
            let msg = String::from_utf8_lossy(&stderr);
            eprintln!(
                "Warning: formatter failed for {}: {}",
                path.display(),
                msg.trim()
            );
        }
        Err(e) => {
            eprintln!(
                "Warning: could not run formatter for {}: {}",
                path.display(),
                e
            );
        }
        _ => {}
    }
}
