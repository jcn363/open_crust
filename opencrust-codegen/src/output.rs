//! Output helper for code generation
use crate::errors::Result;
use std::fs;
use std::path::Path;

/// Write the rendered content to a file at `path`.
///
/// Returns `Ok(())` on success or a `CodegenError`.
pub fn write_to<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, content)?;
    Ok(())
}
