use directories::ProjectDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub metadata: SkillMetadata,
    pub content: String,
    pub active: bool,
}

pub struct SkillManager {
    pub skills: HashMap<String, Skill>,
    pub active_skills: Vec<String>,
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            active_skills: Vec::new(),
        }
    }

    pub fn discover(&mut self) {
        let mut search_paths = Vec::new();

        // Local project paths
        if let Ok(current_dir) = std::env::current_dir() {
            let mut curr = current_dir.as_path();
            loop {
                search_paths.push(curr.join(".opencrust/skills"));
                search_paths.push(curr.join(".claude/skills"));
                search_paths.push(curr.join(".agents/skills"));

                if let Some(parent) = curr.parent() {
                    curr = parent;
                } else {
                    break;
                }

                // Stop at git root if possible (optional optimization)
                if curr.join(".git").exists() {
                    search_paths.push(curr.join(".opencrust/skills"));
                    search_paths.push(curr.join(".claude/skills"));
                    search_paths.push(curr.join(".agents/skills"));
                    break;
                }
            }
        }

        // Global paths
        if let Some(proj_dirs) = ProjectDirs::from("ai", "opencust", "open_crust") {
            search_paths.push(proj_dirs.config_dir().join("skills"));
        }
        if let Some(home) = dirs::home_dir() {
            search_paths.push(home.join(".config/opencrust/skills"));
            search_paths.push(home.join(".claude/skills"));
            search_paths.push(home.join(".agents/skills"));
        }

        for path in search_paths {
            self.load_from_dir(&path);
        }
    }

    fn load_from_dir(&mut self, dir: &Path) {
        if !dir.is_dir() {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let skill_dir = entry.path();
                if skill_dir.is_dir() {
                    let skill_file = skill_dir.join("SKILL.md");
                    if skill_file.exists()
                        && let Ok(content) = fs::read_to_string(&skill_file)
                        && let Some((metadata, body)) = self.parse_skill(&content)
                    {
                        let name = metadata.name.clone();
                        self.skills.insert(
                            name,
                            Skill {
                                metadata,
                                content: body.to_string(),
                                active: true,
                            },
                        );
                    }
                }
            }
        }
    }

    fn parse_skill<'a>(&self, content: &'a str) -> Option<(SkillMetadata, &'a str)> {
        if !content.starts_with("---") {
            return None;
        }

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return None;
        }

        let yaml = parts[1];
        let body = parts[2];

        if let Ok(metadata) = serde_yaml::from_str::<SkillMetadata>(yaml) {
            Some((metadata, body))
        } else {
            None
        }
    }

    pub fn get_available_skills_xml(&self) -> String {
        let mut xml = String::from("<available_skills>\n");
        for skill in self.skills.values().filter(|s| s.active) {
            xml.push_str(&format!(
                "  <skill>\n    <name>{}</name>\n    <description>{}</description>\n  </skill>\n",
                skill.metadata.name, skill.metadata.description
            ));
        }
        xml.push_str("</available_skills>");
        xml
    }

    /// Activate a skill for the current session
    pub fn activate_skill(&mut self, name: &str) -> bool {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.active = true;
            if !self.active_skills.contains(&name.to_string()) {
                self.active_skills.push(name.to_string());
            }
            true
        } else {
            false
        }
    }

    /// Deactivate a skill for the current session
    pub fn deactivate_skill(&mut self, name: &str) -> bool {
        if let Some(skill) = self.skills.get_mut(name) {
            if skill.active {
                skill.active = false;
                self.active_skills.retain(|n| n != name);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get a specific skill by name
    #[allow(dead_code)]
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// Get mutable reference to a specific skill
    #[allow(dead_code)]
    pub fn get_skill_mut(&mut self, name: &str) -> Option<&mut Skill> {
        self.skills.get_mut(name)
    }

    /// List all skills with their statistics (for the browser UI)
    pub fn list_skills_with_stats(&self) -> Vec<(String, String, bool)> {
        self.skills
            .values()
            .map(|s| {
                (
                    s.metadata.name.clone(),
                    s.metadata.description.clone(),
                    s.active,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_skill_manager() -> SkillManager {
        let mut manager = SkillManager::new();
        // Manually add a test skill
        let skill = Skill {
            metadata: SkillMetadata {
                name: "test-skill".to_string(),
                description: "A test skill".to_string(),
            },
            content: "Test instructions".to_string(),
            active: true,
        };
        manager.skills.insert("test-skill".to_string(), skill);
        manager
    }

    #[test]
    fn test_activate_skill() {
        let mut manager = create_test_skill_manager();
        assert!(manager.activate_skill("test-skill"));
        assert!(manager.skills["test-skill"].active);

        // Deactivate
        assert!(manager.deactivate_skill("test-skill"));
        assert!(!manager.skills["test-skill"].active);

        // Non-existent skill
        assert!(!manager.activate_skill("non-existent"));
    }

    #[test]
    fn test_deactivate_skill() {
        let mut manager = create_test_skill_manager();
        // First deactivate
        assert!(manager.deactivate_skill("test-skill"));
        assert!(!manager.skills["test-skill"].active);

        // Already deactivated
        assert!(!manager.deactivate_skill("test-skill"));
    }

    #[test]
    fn test_list_skills_with_stats() {
        let mut manager = create_test_skill_manager();
        manager.deactivate_skill("test-skill");

        let stats = manager.list_skills_with_stats();
        assert_eq!(stats.len(), 1);
        let (name, desc, active) = &stats[0];
        assert_eq!(name, "test-skill");
        assert_eq!(desc, "A test skill");
        assert!(!active);
    }

    #[test]
    fn test_get_skill() {
        let manager = create_test_skill_manager();
        let skill = manager.get_skill("test-skill");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().metadata.name, "test-skill");

        let none = manager.get_skill("non-existent");
        assert!(none.is_none());
    }

    #[test]
    fn test_get_skill_mut() {
        let mut manager = create_test_skill_manager();
        let skill = manager.get_skill_mut("test-skill");
        assert!(skill.is_some());
        skill.unwrap().active = false;
        assert!(!manager.skills["test-skill"].active);
    }
}
