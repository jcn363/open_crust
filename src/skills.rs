use std::fs;
use std::path::Path;
use serde::Deserialize;
use std::collections::HashMap;
use directories::ProjectDirs;

#[derive(Debug, Deserialize, Clone)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
}

pub struct Skill {
    pub metadata: SkillMetadata,
    pub content: String,
}

pub struct SkillManager {
    pub skills: HashMap<String, Skill>,
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
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
                            && let Some((metadata, body)) = self.parse_skill(&content) {
                                let name = metadata.name.clone();
                                self.skills.insert(name, Skill {
                                    metadata,
                                    content: body.to_string(),
                                });
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
        for skill in self.skills.values() {
            xml.push_str(&format!(
                "  <skill>\n    <name>{}</name>\n    <description>{}</description>\n  </skill>\n",
                skill.metadata.name, skill.metadata.description
            ));
        }
        xml.push_str("</available_skills>");
        xml
    }
}
