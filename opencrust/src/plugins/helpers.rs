use std::fs;
use std::path::Path;

#[cfg(unix)]
pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
pub(crate) fn is_executable(_path: &Path) -> bool {
    false
}

pub(crate) fn guess_interpreter(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("sh") => "sh".to_string(),
        Some("py") => "python3".to_string(),
        Some("js") => "node".to_string(),
        Some("ts") => "npx".to_string(),
        Some("rb") => "ruby".to_string(),
        Some("rs") => "rust-script".to_string(),
        _ => {
            // Try reading shebang
            if let Ok(content) = fs::read_to_string(path) {
                if let Some(line) = content.lines().next() {
                    if let Some(interp) = line.strip_prefix("#!") {
                        return interp.trim().to_string();
                    }
                }
            }
            "sh".to_string()
        }
    }
}

pub(crate) fn copy_dir_recursively(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursively(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
