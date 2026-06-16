use super::*;
use std::fs;

fn create_test_skill_manager() -> SkillManager {
    let mut manager = SkillManager::new();
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

fn create_skill_dir(base: &std::path::Path, name: &str, description: &str, content: &str) {
    let dir = base.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("SKILL.md"),
        format!(
            "---\nname: {}\ndescription: {}\n---\n{}",
            name, description, content
        ),
    )
    .unwrap();
}

// ---- Core unit tests ----

#[test]
fn test_activate_skill() {
    let mut manager = create_test_skill_manager();
    assert!(manager.activate_skill("test-skill"));
    assert!(manager.skills["test-skill"].active);
    assert!(manager.deactivate_skill("test-skill"));
    assert!(!manager.skills["test-skill"].active);
    assert!(!manager.activate_skill("non-existent"));
}

#[test]
fn test_deactivate_skill() {
    let mut manager = create_test_skill_manager();
    assert!(manager.deactivate_skill("test-skill"));
    assert!(!manager.skills["test-skill"].active);
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
    assert!(manager.get_skill("non-existent").is_none());
}

#[test]
fn test_get_skill_mut() {
    let mut manager = create_test_skill_manager();
    let skill = manager.get_skill_mut("test-skill");
    assert!(skill.is_some());
    skill.unwrap().active = false;
    assert!(!manager.skills["test-skill"].active);
}

#[test]
fn test_search_paths_includes_global() {
    let paths = SkillManager::search_paths();
    assert!(!paths.is_empty());
}

// ---- Integration tests: parsing ----

#[test]
fn test_parse_skill_full_pipeline() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_parse_skill");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let full_content = "---\nname: my-skill\ndescription: Does cool things\n---\n# My Skill\n\nThis skill does really cool stuff.";
    let skill_file = temp_dir.join("SKILL.md");
    fs::write(&skill_file, full_content).unwrap();

    let manager = SkillManager::new();
    let skill = manager
        .parse_skill_file(&temp_dir)
        .expect("Skill should parse");

    assert_eq!(skill.metadata.name, "my-skill");
    assert_eq!(skill.metadata.description, "Does cool things");
    assert_eq!(
        skill.content,
        "# My Skill\n\nThis skill does really cool stuff."
    );
    assert!(skill.active);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parse_skill_missing_file_returns_none() {
    let manager = SkillManager::new();
    let nonexistent = std::path::Path::new("/tmp/nonexistent_skill/SKILL.md");
    assert!(manager.parse_skill_file(nonexistent).is_none());
}

#[test]
fn test_parse_skill_invalid_yaml_returns_none() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_invalid_yaml");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let skill_file = temp_dir.join("SKILL.md");
    fs::write(&skill_file, "---\nname: broken\ninvalid yaml here").unwrap();

    let manager = SkillManager::new();
    assert!(manager.parse_skill_file(&skill_file).is_none());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parse_skill_no_frontmatter_returns_none() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_no_frontmatter");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let skill_file = temp_dir.join("SKILL.md");
    fs::write(&skill_file, "Just some content without frontmatter").unwrap();

    let manager = SkillManager::new();
    assert!(manager.parse_skill_file(&skill_file).is_none());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_skill_active_by_default_after_creation() {
    let skill = Skill::new(
        SkillMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
        },
        "content".to_string(),
    );
    assert!(skill.active);
}

// ---- Integration tests: discover_changes lifecycle ----

#[test]
fn test_discover_changes_detects_new_skill() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_changes_new");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    create_skill_dir(&temp_dir, "new-skill", "A new skill", "Instructions");

    let mut manager = SkillManager::new();
    let (added, removed, modified) =
        manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));

    assert_eq!(added, vec!["new-skill"]);
    assert!(removed.is_empty());
    assert!(modified.is_empty());
    assert!(manager.skills.contains_key("new-skill"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_discover_changes_detects_modified_skill() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_changes_mod");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    create_skill_dir(&temp_dir, "mod-skill", "Original desc", "Original content");

    let mut manager = SkillManager::new();
    // First discovery
    let (added, _, _) = manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));
    assert_eq!(added, vec!["mod-skill"]);

    // Deactivate before modifying on disk
    manager.deactivate_skill("mod-skill");
    assert!(!manager.skills["mod-skill"].active);

    // Overwrite the file with new content/description
    let skill_dir = temp_dir.join("mod-skill");
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: mod-skill\ndescription: Updated desc\n---\nUpdated content",
    )
    .unwrap();

    // Re-discover
    let (added, removed, modified) =
        manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));

    assert!(added.is_empty());
    assert!(removed.is_empty());
    assert_eq!(modified, vec!["mod-skill"]);
    assert_eq!(manager.skills["mod-skill"].content, "Updated content");
    assert_eq!(
        manager.skills["mod-skill"].metadata.description,
        "Updated desc"
    );
    // Deactivation state must survive modification
    assert!(!manager.skills["mod-skill"].active);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_discover_changes_detects_deleted_skill() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_changes_del");
    let _ = fs::remove_dir_all(&temp_dir);

    fs::create_dir_all(&temp_dir).unwrap();
    create_skill_dir(&temp_dir, "to-delete", "Will be deleted", "Some content");

    let mut manager = SkillManager::new();
    // First discovery
    let (added, _, _) = manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));
    assert_eq!(added, vec!["to-delete"]);

    // Remove the skill directory
    fs::remove_dir_all(temp_dir.join("to-delete")).unwrap();

    // Re-discover
    let (added, removed, modified) =
        manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));

    assert!(added.is_empty());
    assert_eq!(removed, vec!["to-delete"]);
    assert!(modified.is_empty());
    assert!(!manager.skills.contains_key("to-delete"));

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_discover_changes_preserves_active_state_of_unmodified_skills() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_changes_preserve");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    create_skill_dir(&temp_dir, "persist-active", "Test", "Content");

    let mut manager = SkillManager::new();
    manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));
    manager.deactivate_skill("persist-active");
    assert!(!manager.skills["persist-active"].active);

    // Re-discover with no changes on disk
    let (added, removed, modified) =
        manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));

    assert!(added.is_empty());
    assert!(removed.is_empty());
    assert!(modified.is_empty());
    assert!(!manager.skills["persist-active"].active);

    let _ = fs::remove_dir_all(&temp_dir);
}

// ---- Integration tests: XML generation for LLM ----

#[test]
fn test_get_available_skills_xml_includes_active_only() {
    let mut manager = SkillManager::new();
    manager.skills.insert(
        "active-skill".to_string(),
        Skill {
            metadata: SkillMetadata {
                name: "active-skill".to_string(),
                description: "I am active".to_string(),
            },
            content: "active".to_string(),
            active: true,
        },
    );
    manager.skills.insert(
        "inactive-skill".to_string(),
        Skill {
            metadata: SkillMetadata {
                name: "inactive-skill".to_string(),
                description: "I am inactive".to_string(),
            },
            content: "inactive".to_string(),
            active: false,
        },
    );

    let xml = manager.get_available_skills_xml();
    assert!(xml.contains("<available_skills>"));
    assert!(xml.contains("active-skill"));
    assert!(xml.contains("I am active"));
    assert!(!xml.contains("inactive-skill"));
    assert!(!xml.contains("I am inactive"));
}

#[test]
fn test_get_available_skills_xml_empty_when_no_skills() {
    let manager = SkillManager::new();
    let xml = manager.get_available_skills_xml();
    assert!(xml.contains("<available_skills>"));
    assert!(xml.contains("</available_skills>"));
    assert_eq!(xml.lines().count(), 2);
}

#[test]
fn test_full_pipeline_from_disk_to_xml() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_full_pipeline");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    create_skill_dir(
        &temp_dir,
        "python-skill",
        "Helps with Python",
        "Write Python code",
    );
    create_skill_dir(
        &temp_dir,
        "rust-skill",
        "Helps with Rust",
        "Write Rust code",
    );

    let mut manager = SkillManager::new();
    let (added, _, _) = manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));
    assert_eq!(added.len(), 2);
    assert!(manager.skills.contains_key("python-skill"));
    assert!(manager.skills.contains_key("rust-skill"));

    manager.deactivate_skill("rust-skill");

    let xml = manager.get_available_skills_xml();
    assert!(xml.contains("python-skill"));
    assert!(xml.contains("Helps with Python"));
    assert!(!xml.contains("rust-skill"));

    let _ = fs::remove_dir_all(&temp_dir);
}

// ---- Integration tests: UI browser items sync ----

#[test]
fn test_skill_browser_items_sync() {
    let temp_dir = std::env::temp_dir().join("opencrust_test_browser_sync");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    create_skill_dir(
        &temp_dir,
        "browser-test",
        "Browser test skill",
        "Test content",
    );

    let mut manager = SkillManager::new();
    manager.discover_changes_with_path(std::slice::from_ref(&temp_dir));

    let stats = manager.list_skills_with_stats();
    assert_eq!(stats.len(), 1);
    let (name, desc, active) = &stats[0];
    assert_eq!(name, "browser-test");
    assert_eq!(desc, "Browser test skill");
    assert!(active);

    let _ = fs::remove_dir_all(&temp_dir);
}
