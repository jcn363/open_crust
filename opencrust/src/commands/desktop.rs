use crate::cli::DesktopCommands;
use crate::desktop::detection::{
    detect_desktop, detect_display_server, get_cinnamon_info, is_supported_desktop,
};
use crate::desktop::file_picker::{
    FilePickerMode, FilePickerOptions, detect_file_picker_backend, file_picker,
    is_file_picker_available,
};
use crate::desktop::notifications::{
    Notification, NotificationUrgency, is_notification_available, notify_error, notify_success,
    send_notification_smart,
};
use crate::error::Result;

pub async fn handle_desktop(cmd: DesktopCommands) -> Result<()> {
    match cmd {
        DesktopCommands::FilePicker { mode, dir, title } => {
            if !is_file_picker_available() {
                eprintln!(
                    "Error: No file picker backend available (need nemo, zenity, or kdialog)"
                );
                return Ok(());
            }
            let backend = detect_file_picker_backend();

            let mode = match mode.as_str() {
                "open" => FilePickerMode::OpenFile,
                "open-multiple" => FilePickerMode::OpenMultiple,
                "save" => FilePickerMode::Save,
                "directory" => FilePickerMode::Directory,
                _ => {
                    eprintln!(
                        "Invalid mode: {}. Use: open, open-multiple, save, directory",
                        mode
                    );
                    return Ok(());
                }
            };

            let options = FilePickerOptions {
                initial_dir: dir.as_ref().map(std::path::PathBuf::from),
                title: title.clone(),
                ..Default::default()
            };

            let result = file_picker(mode, &options);
            if result.cancelled {
                println!("Cancelled");
            } else {
                for path in result.paths {
                    println!("{}", path.display());
                }
            }
            println!("Backend: {}", backend.name());
        }
        DesktopCommands::Notify {
            title,
            body,
            urgency,
        } => {
            if !is_notification_available() {
                eprintln!("Warning: No notification daemon available");
            }

            let urgency = NotificationUrgency::from_name(&urgency);
            let notification = Notification::new(&title, &body)
                .with_urgency(urgency)
                .with_expire_timeout(10);

            match send_notification_smart(&notification) {
                Ok(_) => {
                    println!("Notification sent: {} - {}", title, body);
                    let _ = notify_success("Notification sent", format!("{} - {}", title, body));
                }
                Err(e) => {
                    eprintln!("Failed to send notification: {}", e);
                    let _ = notify_error("Notification failed", e.to_string());
                }
            }
        }
        DesktopCommands::Detect => {
            let desktop = detect_desktop();
            let display_server = detect_display_server();
            println!("Desktop environment: {}", desktop);
            println!("Display server: {}", display_server.name());
            println!("Supported: {}", is_supported_desktop());

            if desktop.is_cinnamon() {
                let info = get_cinnamon_info();
                println!("\nCinnamon Info:");
                println!(
                    "  Version: {}",
                    info.version.as_deref().unwrap_or("unknown")
                );
                println!(
                    "  Theme: background={}, foreground={}",
                    info.theme.background, info.theme.foreground
                );
                println!("  Accent: {}", info.theme.accent);
                println!("  Icon theme: {}", info.icon_theme);
                println!("  Cursor theme: {}", info.cursor_theme);
                println!("  Display server: {}", info.display_server.name());
            }
        }
    }
    Ok(())
}
