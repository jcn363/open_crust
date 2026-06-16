use crate::cli::SkillsCommands;
use crate::skills::SkillManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_skills(
    cmd: SkillsCommands,
    skill_manager: Arc<Mutex<SkillManager>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut skills = skill_manager.lock().await;
    match cmd {
        SkillsCommands::List => {
            println!("Skills (use 'opencrust skills activate/deactivate <name>' to toggle):");
            println!();
            for (name, description, active) in skills.list_skills_with_stats() {
                let status = if active { "ACTIVE" } else { "inactive" };
                println!("[{}] {} - {}", status, name, description);
            }
        }
        SkillsCommands::Activate { name } => {
            if skills.activate_skill(&name) {
                println!("Skill '{}' activated.", name);
            } else {
                eprintln!("Skill '{}' not found.", name);
            }
        }
        SkillsCommands::Deactivate { name } => {
            if skills.deactivate_skill(&name) {
                println!("Skill '{}' deactivated.", name);
            } else {
                eprintln!("Skill '{}' not found.", name);
            }
        }
    }
    Ok(())
}
