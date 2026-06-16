//! Skill system: file-based skill discovery, activation, and XML generation
//!
//! Skills are Markdown files with YAML frontmatter stored on disk. This module
//! discovers skills from multiple search paths, parses them, tracks active/
//! inactive state, generates XML for LLM system prompts, and supports hot-reload
//! without restart via periodic change detection.

use directories::ProjectDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

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

impl Skill {
    fn new(metadata: SkillMetadata, content: String) -> Self {
        Self {
            metadata,
            content,
            active: true,
        }
    }
}

/// Manages skill discovery, activation, and XML generation
///
/// Discovers skills from multiple search paths, parses YAML frontmatter,
/// tracks active/inactive state, detects disk changes for hot-reload,
/// and generates XML for LLM system prompts.
pub struct SkillManager {
    pub skills: HashMap<String, Skill>,
    pub active_skills: Vec<String>,
    last_discovery: Instant,
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            active_skills: Vec::new(),
            last_discovery: Instant::now(),
        }
    }

    /// Collect all skill search paths, in priority order (local first, then global).
    fn search_paths() -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();

        if let Ok(current_dir) = std::env::current_dir() {
            let mut curr = current_dir.as_path();
            loop {
                paths.push(curr.join(".opencrust/skills"));
                paths.push(curr.join(".claude/skills"));
                paths.push(curr.join(".agents/skills"));

                if let Some(parent) = curr.parent() {
                    curr = parent;
                } else {
                    break;
                }

                if curr.join(".git").exists() {
                    paths.push(curr.join(".opencrust/skills"));
                    paths.push(curr.join(".claude/skills"));
                    paths.push(curr.join(".agents/skills"));
                    break;
                }
            }
        }

        if let Some(proj_dirs) = ProjectDirs::from("ai", "opencust", "opencrust") {
            paths.push(proj_dirs.config_dir().join("skills"));
        }
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".config/opencrust/skills"));
            paths.push(home.join(".claude/skills"));
            paths.push(home.join(".agents/skills"));
        }

        paths
    }

    /// Scan all search paths and load/reload every skill.
    pub fn discover(&mut self) {
        for path in Self::search_paths() {
            self.load_from_dir(&path);
        }
        self.last_discovery = Instant::now();
    }

    /// Full re-discovery: detects new, modified, and deleted skills.
    /// Preserves deactivation state of existing, unmodified skills.
    /// Returns (added, removed, modified) skill names.
    pub fn discover_changes(&mut self) -> (Vec<String>, Vec<String>, Vec<String>) {
        self.discover_changes_with_path(&Self::search_paths())
    }

    /// Like discover_changes, but accepts custom paths for isolated testing.
    fn discover_changes_with_path(
        &mut self,
        paths: &[std::path::PathBuf],
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut seen_names: HashMap<String, Skill> = HashMap::new();

        for path in paths {
            if !path.is_dir() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let skill_dir = entry.path();
                    if skill_dir.is_dir()
                        && let Some(skill) = self.parse_skill_file(&skill_dir)
                    {
                        let name = skill.metadata.name.clone();
                        if let Some(existing) = self.skills.get(&name) {
                            // Check if content or description changed
                            if existing.content != skill.content
                                || existing.metadata.description != skill.metadata.description
                            {
                                let was_active = existing.active;
                                self.skills.insert(
                                    name.clone(),
                                    Skill {
                                        metadata: skill.metadata,
                                        content: skill.content,
                                        active: was_active,
                                    },
                                );
                                modified.push(name.clone());
                            }
                        } else {
                            self.skills.insert(name.clone(), skill);
                            added.push(name.clone());
                        }
                        seen_names.insert(name.clone(), self.skills[&name].clone());
                    }
                }
            }
        }

        // Detect removed skills: skills in memory but no longer on disk
        let mut removed = Vec::new();
        self.skills.retain(|name, _skill| {
            if seen_names.contains_key(name) {
                true
            } else {
                removed.push(name.clone());
                false
            }
        });

        // Also clean up active_skills for removed skills
        self.active_skills
            .retain(|name| self.skills.contains_key(name));

        self.last_discovery = Instant::now();
        (added, removed, modified)
    }

    /// Check if it's time to poll for new skills (every 30 seconds).
    pub fn should_check_for_updates(&self) -> bool {
        self.last_discovery.elapsed() >= Duration::from_secs(30)
    }

    fn load_from_dir(&mut self, dir: &Path) {
        if !dir.is_dir() {
            return;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let skill_dir = entry.path();
                if skill_dir.is_dir()
                    && let Some(skill) = self.parse_skill_file(&skill_dir)
                {
                    self.skills.insert(skill.metadata.name.clone(), skill);
                }
            }
        }
    }

    /// Parse a SKILL.md file from a skill directory.
    fn parse_skill_file(&self, skill_dir: &std::path::Path) -> Option<Skill> {
        let skill_file = skill_dir.join("SKILL.md");
        if !skill_file.exists() {
            return None;
        }
        let content = fs::read_to_string(&skill_file).ok()?;
        self.parse_skill(&content)
            .map(|(metadata, body)| Skill::new(metadata, body.to_string()))
    }

    fn parse_skill<'a>(&self, content: &'a str) -> Option<(SkillMetadata, &'a str)> {
        // Parse front‑matter delimited by "---".
        if !content.starts_with("---") {
            return None;
        }
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return None;
        }
        let yaml = parts[1].trim_start_matches('\n');
        let body = parts[2].trim_start_matches('\n');
        serde_yaml::from_str::<SkillMetadata>(yaml)
            .ok()
            .map(|metadata| (metadata, body))
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

    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "used in tests; dead in non-test builds")
    )]
    pub fn get_skill_mut(&mut self, name: &str) -> Option<&mut Skill> {
        self.skills.get_mut(name)
    }

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
mod tests;
