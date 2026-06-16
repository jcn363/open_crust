use crate::app::{App, Message};
use crate::skills::SkillManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Check for skill updates and apply changes
pub async fn check_skill_updates(
    app: &mut App,
    skill_manager: &Arc<Mutex<SkillManager>>,
) {
    let mut skills = skill_manager.lock().await;
    if skills.should_check_for_updates() {
        let (added, removed, modified) = skills.discover_changes();
        if !added.is_empty() {
            app.tabs[0].messages.push(Message::new(format!(
                "System: Discovered new skills: {}",
                added.join(", ")
            )));
            for name in &added {
                if let Some(skill) = skills.get_skill(name) {
                    app.skill_browser_items.push((
                        skill.metadata.name.clone(),
                        skill.metadata.description.clone(),
                        skill.active,
                    ));
                }
            }
            app.mark_dirty();
        }
        if !removed.is_empty() {
            app.tabs[0].messages.push(Message::new(format!(
                "System: Removed skills: {}",
                removed.join(", ")
            )));
            app.skill_browser_items
                .retain(|(name, _, _)| !removed.contains(name));
            app.mark_dirty();
        }
        if !modified.is_empty() {
            app.tabs[0].messages.push(Message::new(format!(
                "System: Modified skills: {}",
                modified.join(", ")
            )));
            for name in &modified {
                if let Some(skill) = skills.get_skill(name) {
                    // Remove stale entry first, then re-add with updated data
                    app.skill_browser_items.retain(|(n, _, _)| n != name);
                    app.skill_browser_items.push((
                        skill.metadata.name.clone(),
                        skill.metadata.description.clone(),
                        skill.active,
                    ));
                }
            }
            app.mark_dirty();
        }
    }
}