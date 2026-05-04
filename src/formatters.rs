use std::process::Command;
use std::path::Path;

pub fn format_file(path: &Path) {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    match extension {
        "rs" => {
            let _ = Command::new("rustfmt")
                .arg(path)
                .spawn()
                .and_then(|mut child| child.wait());
        }
        "js" | "ts" | "json" | "md" => {
            let _ = Command::new("prettier")
                .arg("--write")
                .arg(path)
                .spawn()
                .and_then(|mut child| child.wait());
        }
        _ => {}
    }
}
