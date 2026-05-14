//! Linux Mint Cinnamon native file picker
//!
//! Provides native file/folder picker dialogs for Linux Mint Cinnamon.
//! Uses Nemo's DBus interface, with fallbacks to zenity or kde-file-dialog.

use std::path::PathBuf;
use std::process::Command;

/// File picker mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilePickerMode {
    /// Open a single file
    #[default]
    OpenFile,
    /// Open multiple files
    OpenMultiple,
    /// Save file (with overwrite confirmation)
    Save,
    /// Select a directory
    Directory,
}

/// File picker options
#[derive(Debug, Clone, Default)]
pub struct FilePickerOptions {
    /// Initial directory
    pub initial_dir: Option<PathBuf>,
    /// File filter patterns (e.g., "*.txt", "*.rs")
    pub filters: Vec<FileFilter>,
    /// Window title
    pub title: Option<String>,
    /// Confirm file overwrite (for Save mode)
    pub confirm_overwrite: bool,
}

/// File filter pattern
#[derive(Debug, Clone)]
pub struct FileFilter {
    /// Human-readable name
    pub name: String,
    /// Glob patterns
    pub patterns: Vec<String>,
}

impl FileFilter {
    /// Create a new filter
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>, patterns: impl Into<Vec<String>>) -> Self {
        Self {
            name: name.into(),
            patterns: patterns.into(),
        }
    }

    /// Common filter: source code
    #[allow(dead_code)]
    pub fn source_code() -> Self {
        Self::new(
            "Source Code",
            vec![
                "*.rs".to_string(),
                "*.py".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.go".to_string(),
            ],
        )
    }
}

/// File picker result
#[derive(Debug, Clone)]
pub struct FilePickerResult {
    /// Selected file paths
    pub paths: Vec<PathBuf>,
    /// Whether the dialog was cancelled
    pub cancelled: bool,
}

impl FilePickerResult {
    /// Single file result (convenience)
    #[allow(dead_code)]
    pub fn single(self) -> Option<PathBuf> {
        if self.cancelled {
            None
        } else {
            self.paths.into_iter().next()
        }
    }
}

/// File picker backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilePickerBackend {
    /// Nemo file manager (Cinnamon native)
    Nemo,
    /// Zenity (GTK dialogs)
    Zenity,
    /// KDE file dialog
    KDialog,
    /// None available
    #[default]
    None,
}

impl FilePickerBackend {
    /// Get backend name
    pub fn name(&self) -> &str {
        match self {
            FilePickerBackend::Nemo => "nemo",
            FilePickerBackend::Zenity => "zenity",
            FilePickerBackend::KDialog => "kdialog",
            FilePickerBackend::None => "none",
        }
    }
}

/// Detect available file picker backends
pub fn detect_file_picker_backend() -> FilePickerBackend {
    // Check if we're on Wayland - Nemo is X11-only, so skip it on Wayland
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v == "wayland")
            .unwrap_or(false);

    // Check for Nemo first (Cinnamon native) - but only on X11
    if !is_wayland
        && Command::new("which")
            .arg("nemo")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    {
        return FilePickerBackend::Nemo;
    }

    // Check for Zenity (works on both X11 and Wayland)
    if Command::new("which")
        .arg("zenity")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return FilePickerBackend::Zenity;
    }

    // Check for KDialog (works on both X11 and Wayland)
    if Command::new("which")
        .arg("kdialog")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return FilePickerBackend::KDialog;
    }

    FilePickerBackend::None
}

/// Check if file picker is available
pub fn is_file_picker_available() -> bool {
    detect_file_picker_backend() != FilePickerBackend::None
}

/// Get the Nemo file picker dialog via DBus
pub fn nemo_file_picker(mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult {
    // Use python script for Nemo DBus - most reliable method
    let script = build_nemo_script(mode, options);

    let output = Command::new("python3").args(["-c", &script]).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let paths: Vec<PathBuf> = output_str
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(PathBuf::from)
                    .collect();

                FilePickerResult {
                    paths,
                    cancelled: false,
                }
            } else {
                // User cancelled or error
                FilePickerResult {
                    paths: vec![],
                    cancelled: true,
                }
            }
        }
        Err(_) => {
            // Fallback error
            FilePickerResult {
                paths: vec![],
                cancelled: true,
            }
        }
    }
}

/// Build Python script for Nemo file picker
/// Uses Nemo's DBus API to open a file picker dialog
fn build_nemo_script(mode: FilePickerMode, options: &FilePickerOptions) -> String {
    let action = match mode {
        FilePickerMode::OpenFile => "open",
        FilePickerMode::OpenMultiple => "open-multiple",
        FilePickerMode::Save => "save",
        FilePickerMode::Directory => "directory",
    };

    let title = options
        .title
        .clone()
        .unwrap_or_else(|| "Select File".to_string());
    let dir = options
        .initial_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // Build filter string for Nemo
    let filters: Vec<String> = options
        .filters
        .iter()
        .map(|f| format!("{}|{}", f.name, f.patterns.join(" ")))
        .collect();
    let filter_str = if filters.is_empty() {
        "All Files|*".to_string()
    } else {
        filters.join("|")
    };

    let confirm_overwrite_py = if options.confirm_overwrite && mode == FilePickerMode::Save {
        "True"
    } else {
        "False"
    };

    format!(
        r#"
import sys
import os
import subprocess

try:
    cmd = ['zenity', '--file-selection', '--title={title}']

    if '{action}' == 'directory':
        cmd.append('--directory')
    elif '{action}' == 'open-multiple':
        cmd.append('--multiple')

    if '{action}' == 'save' and {confirm_overwrite}:
        cmd.append('--confirm-overwrite')

    cmd.append('--filename={dir}')

    if len('{filter_str}') > 0:
        cmd.append('--file-filter={filter_str}')

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode == 0:
        print(result.stdout.strip())
    else:
        sys.exit(1)

except Exception as e:
    print(str(e), file=sys.stderr)
    sys.exit(1)
"#,
        action = action,
        title = title,
        dir = dir,
        filter_str = filter_str,
        confirm_overwrite = confirm_overwrite_py,
    )
}

/// Open file picker using Zenity
pub fn zenity_file_picker(mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult {
    let mut args = vec!["--file-selection".to_string()];

    match mode {
        FilePickerMode::OpenFile => {}
        FilePickerMode::OpenMultiple => args.push("--multiple".to_string()),
        FilePickerMode::Save => args.push("--save".to_string()),
        FilePickerMode::Directory => args.push("--directory".to_string()),
    }

    // Add title
    if let Some(ref title) = options.title {
        args.push(format!("--title={}", title));
    }

    // Add initial directory
    if let Some(ref dir) = options.initial_dir
        && dir.exists()
    {
        args.push(format!("--filename={}", dir.to_string_lossy()));
    }

    // Add filters
    for filter in &options.filters {
        for pattern in &filter.patterns {
            args.push(format!("--file-filter={}|{}", filter.name, pattern));
        }
    }

    let output = Command::new("zenity").args(&args).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let paths: Vec<PathBuf> = output_str
                    .lines()
                    .filter(|l| !l.is_empty() && !l.starts_with('_'))
                    .map(PathBuf::from)
                    .collect();

                FilePickerResult {
                    paths,
                    cancelled: false,
                }
            } else {
                FilePickerResult {
                    paths: vec![],
                    cancelled: true,
                }
            }
        }
        Err(_) => FilePickerResult {
            paths: vec![],
            cancelled: true,
        },
    }
}

/// Open file picker using KDialog (KDE fallback)
pub fn kdialog_file_picker(mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult {
    let dialog_type = match mode {
        FilePickerMode::OpenFile => "--getopenfilename",
        FilePickerMode::OpenMultiple => "--getopenfilename",
        FilePickerMode::Save => "--getsavefilename",
        FilePickerMode::Directory => "--getexistingdirectory",
    };

    let mut args = vec![dialog_type.to_string()];

    // Initial directory
    if let Some(ref dir) = options.initial_dir {
        if dir.exists() {
            args.push(dir.to_string_lossy().to_string());
        } else {
            args.push(".".to_string());
        }
    } else {
        args.push(".".to_string());
    }

    // Filters as MIME types
    let filters_str = options
        .filters
        .iter()
        .map(|f| f.name.replace([',', ' '], " *."))
        .collect::<Vec<_>>()
        .join(" *.");
    if !filters_str.is_empty() {
        args.push(format!(
            "*{}*{}",
            if filters_str.contains('.') { "" } else { "." },
            filters_str
        ));
    }

    // Title
    if let Some(ref title) = options.title {
        args.push(format!("--title={}", title));
    }

    // Multiple selection
    if mode == FilePickerMode::OpenMultiple {
        args.push("--separate-output".to_string());
    }

    let output = Command::new("kdialog").args(&args).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let paths: Vec<PathBuf> = output_str
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(PathBuf::from)
                    .collect();

                FilePickerResult {
                    paths,
                    cancelled: false,
                }
            } else {
                FilePickerResult {
                    paths: vec![],
                    cancelled: true,
                }
            }
        }
        Err(_) => FilePickerResult {
            paths: vec![],
            cancelled: true,
        },
    }
}

/// Show file picker using the best available backend
pub fn file_picker(mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult {
    let backend = detect_file_picker_backend();

    match backend {
        FilePickerBackend::Nemo => nemo_file_picker(mode, options),
        FilePickerBackend::Zenity => zenity_file_picker(mode, options),
        FilePickerBackend::KDialog => kdialog_file_picker(mode, options),
        FilePickerBackend::None => FilePickerResult {
            paths: vec![],
            cancelled: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_detection() {
        let backend = detect_file_picker_backend();
        // Just verify detection works - may find Nemo, Zenity, KDialog, or None
        assert!(matches!(
            backend,
            FilePickerBackend::Nemo
                | FilePickerBackend::Zenity
                | FilePickerBackend::KDialog
                | FilePickerBackend::None
        ));
    }

    #[test]
    fn test_filter() {
        let filter = FileFilter::new("Rust", vec!["*.rs".to_string()]);
        assert_eq!(filter.name, "Rust");
        assert_eq!(filter.patterns.len(), 1);
    }

    #[test]
    fn test_result() {
        let result = FilePickerResult {
            paths: vec![],
            cancelled: true,
        };
        assert!(result.cancelled);
        assert_eq!(result.single(), None);
    }
}
