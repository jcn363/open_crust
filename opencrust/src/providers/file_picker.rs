//! File picker provider trait
//!
//! Abstracts native file/folder picker dialogs across platforms.

use crate::desktop::file_picker::{FilePickerMode, FilePickerOptions, FilePickerResult};
use crate::providers::Provider;
use std::path::PathBuf;

/// File picker provider trait for native file dialogs
pub trait FilePickerProvider: Provider {
    /// Open a file picker dialog
    fn pick(&self, mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult;

    /// Convenience: pick a single file
    fn pick_file(&self, options: &FilePickerOptions) -> Option<PathBuf> {
        self.pick(FilePickerMode::OpenFile, options).single()
    }

    /// Convenience: pick multiple files
    fn pick_files(&self, options: &FilePickerOptions) -> Vec<PathBuf> {
        self.pick(FilePickerMode::OpenMultiple, options).paths
    }

    /// Convenience: pick a directory
    fn pick_directory(&self, options: &FilePickerOptions) -> Option<PathBuf> {
        self.pick(FilePickerMode::Directory, options).single()
    }

    /// Convenience: save file dialog
    fn save_file(&self, options: &FilePickerOptions) -> Option<PathBuf> {
        self.pick(FilePickerMode::Save, options).single()
    }
}

/// Zenity file picker provider (Linux GTK)
pub struct ZenityFilePickerProvider;

impl Provider for ZenityFilePickerProvider {
    fn id(&self) -> &str {
        "zenity"
    }

    fn name(&self) -> &str {
        "Zenity (GTK)"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("which")
            .arg("zenity")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn priority(&self) -> u8 {
        70
    }
}

impl FilePickerProvider for ZenityFilePickerProvider {
    fn pick(&self, mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult {
        crate::desktop::file_picker::zenity_file_picker(mode, options)
    }
}

/// KDialog file picker provider (KDE)
pub struct KDialogFilePickerProvider;

impl Provider for KDialogFilePickerProvider {
    fn id(&self) -> &str {
        "kdialog"
    }

    fn name(&self) -> &str {
        "KDialog (KDE)"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("which")
            .arg("kdialog")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn priority(&self) -> u8 {
        60
    }
}

impl FilePickerProvider for KDialogFilePickerProvider {
    fn pick(&self, mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult {
        crate::desktop::file_picker::kdialog_file_picker(mode, options)
    }
}

/// Nemo file picker provider (Cinnamon)
pub struct NemoFilePickerProvider;

impl Provider for NemoFilePickerProvider {
    fn id(&self) -> &str {
        "nemo"
    }

    fn name(&self) -> &str {
        "Nemo (Cinnamon)"
    }

    fn is_available(&self) -> bool {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
            || std::env::var("XDG_SESSION_TYPE")
                .map(|v| v == "wayland")
                .unwrap_or(false);
        if is_wayland {
            return false;
        }
        std::process::Command::new("which")
            .arg("nemo")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn priority(&self) -> u8 {
        80
    }
}

impl FilePickerProvider for NemoFilePickerProvider {
    fn pick(&self, mode: FilePickerMode, options: &FilePickerOptions) -> FilePickerResult {
        crate::desktop::file_picker::nemo_file_picker(mode, options)
    }
}

/// Registry for file picker providers
pub type FilePickerProviderRegistry = crate::providers::ProviderRegistry<dyn FilePickerProvider>;

/// Create default file picker provider registry
pub fn default_file_picker_registry() -> FilePickerProviderRegistry {
    let mut registry = FilePickerProviderRegistry::new();
    registry.register(Box::new(NemoFilePickerProvider));
    registry.register(Box::new(ZenityFilePickerProvider));
    registry.register(Box::new(KDialogFilePickerProvider));
    registry
}
