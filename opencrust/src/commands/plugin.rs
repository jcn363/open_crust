use crate::cli::PluginCommands;
use crate::error::Result;
use crate::plugins::PluginManager;

pub async fn handle_plugin(cmd: PluginCommands) -> Result<()> {
    let mut plugin_mgr = PluginManager::new();
    plugin_mgr.discover();
    match cmd {
        PluginCommands::List => {
            let plugins = plugin_mgr.list();
            if plugins.is_empty() {
                println!("No plugins discovered.");
                println!("\nSearch paths:");
                println!("  - .opencrust/plugins/");
                if let Some(config_dir) = dirs::config_dir() {
                    println!("  - {}", config_dir.join("opencrust/plugins").display());
                }
            } else {
                println!(
                    "{:<24} {:<10} {:<8} {:<40}",
                    "Name", "Version", "Status", "Description"
                );
                println!("{}", "-".repeat(90));
                for p in &plugins {
                    let status = if p.enabled { "enabled" } else { "disabled" };
                    let desc = if p.description.len() > 38 {
                        format!("{}...", &p.description[..35])
                    } else {
                        p.description.clone()
                    };
                    println!(
                        "{:<24} {:<10} {:<8} {:<40}",
                        p.name, p.version, status, desc
                    );
                }
            }
        }
        PluginCommands::Show { name } => match plugin_mgr.get(&name) {
            Some(p) => {
                println!("Name:        {}", p.name);
                println!("Version:     {}", p.version);
                println!("Description: {}", p.description);
                println!("Author:      {}", p.author);
                println!("Path:        {}", p.path.display());
                println!("Enabled:     {}", p.enabled);
                println!("Entry:       {}", p.entry.as_deref().unwrap_or("(none)"));
                println!("Hooks:       {}", p.hooks.join(", "));
                println!("Tools:       {}", p.tools.join(", "));
                println!("Deps:        {}", p.dependencies.join(", "));
            }
            None => eprintln!("Plugin '{}' not found.", name),
        },
        PluginCommands::Install { path } => {
            let src = std::path::PathBuf::from(&path);
            if !src.exists() {
                eprintln!("Error: path '{}' does not exist", path);
                return Ok(());
            }
            match plugin_mgr.install(&src) {
                Ok(name) => println!("Plugin '{}' installed successfully.", name),
                Err(e) => eprintln!("Error installing plugin: {}", e),
            }
        }
        PluginCommands::Remove { name } => match plugin_mgr.remove(&name) {
            Ok(_) => println!("Plugin '{}' removed.", name),
            Err(e) => eprintln!("Error removing plugin: {}", e),
        },
        PluginCommands::Enable { name } => match plugin_mgr.enable(&name) {
            Ok(_) => println!("Plugin '{}' enabled.", name),
            Err(e) => eprintln!("Error enabling plugin: {}", e),
        },
        PluginCommands::Disable { name } => match plugin_mgr.disable(&name) {
            Ok(_) => println!("Plugin '{}' disabled.", name),
            Err(e) => eprintln!("Error disabling plugin: {}", e),
        },
        PluginCommands::Stats => {
            let stats = plugin_mgr.stats();
            println!("{}", stats);
        }
    }
    Ok(())
}
