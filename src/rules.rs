use std::fs;
use std::path::Path;
use std::env;

pub fn load_rules(instructions: &[String]) -> String {
    let mut rules = String::new();

    // 1. External Instructions (from config)
    for pattern in instructions {
        if let Ok(entries) = glob::glob(pattern) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    rules.push_str(&format!("\n\n### Instruction: {}\n", entry.display()));
                    rules.push_str(&content);
                }
            }
        } else if Path::new(pattern).exists() {
            if let Ok(content) = fs::read_to_string(pattern) {
                rules.push_str(&format!("\n\n### Instruction: {}\n", pattern));
                rules.push_str(&content);
            }
        }
    }

    // 2. Project Rules (Traverse up from current dir)
    if let Ok(current_dir) = env::current_dir() {
        let mut dir = current_dir;
        loop {
            let agents_md = dir.join("AGENTS.md");
            let claude_md = dir.join("CLAUDE.md");

            if agents_md.exists() {
                if let Ok(content) = fs::read_to_string(agents_md) {
                    rules.push_str("\n\n### Project Rules (AGENTS.md)\n");
                    rules.push_str(&content);
                    break;
                }
            } else if claude_md.exists() {
                if let Ok(content) = fs::read_to_string(claude_md) {
                    rules.push_str("\n\n### Project Rules (CLAUDE.md fallback)\n");
                    rules.push_str(&content);
                    break;
                }
            }

            if let Some(parent) = dir.parent() {
                dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }

    // 3. Global Rules (~/.config/open_crust/AGENTS.md)
    if let Some(user_dirs) = directories::UserDirs::new() {
        let home = user_dirs.home_dir();
        let global_rules = home.join(".config").join("open_crust").join("AGENTS.md");
        if global_rules.exists() {
            if let Ok(content) = fs::read_to_string(global_rules) {
                rules.push_str("\n\n### Global Rules\n");
                rules.push_str(&content);
            }
        }
    }

    rules
}

pub fn init_project_rules() -> Result<String, String> {
    let path = Path::new("AGENTS.md");
    if path.exists() {
        return Ok("AGENTS.md already exists.".to_string());
    }

    let template = "# Project Rules\n\n## Build Commands\n- `cargo build`\n\n## Test Commands\n- `cargo test`\n";
    fs::write(path, template).map_err(|e| e.to_string())?;
    Ok("Created AGENTS.md template.".to_string())
}
