//! Workflows System for creating and executing parameterized command workflows
//! Inspired by Warp Terminal's Workflows feature

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// A workflow parameter that can be customized when executing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    /// Parameter name (used in template as {{name}})
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Default value if not provided
    pub default_value: Option<String>,
    /// Whether this parameter is required
    pub required: bool,
    /// Parameter type for validation
    pub param_type: ParameterType,
    /// Possible values (for enum-like parameters)
    pub possible_values: Option<Vec<String>>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterType {
    /// String parameter
    String,
    /// Number parameter
    Number,
    /// Boolean parameter
    Boolean,
    /// File path parameter
    FilePath,
    /// Directory path parameter
    DirPath,
    /// Choice from predefined values
    Choice,
}

impl ParameterType {
    /// Get icon for display
    pub fn icon(&self) -> &str {
        match self {
            ParameterType::String => "📝",
            ParameterType::Number => "🔢",
            ParameterType::Boolean => "✅",
            ParameterType::FilePath => "📄",
            ParameterType::DirPath => "📁",
            ParameterType::Choice => "📋",
        }
    }
}

/// A workflow template with parameterized commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this workflow does
    pub description: String,
    /// Workflow category for organization
    pub category: WorkflowCategory,
    /// Template commands with {{parameter}} placeholders
    pub commands: Vec<String>,
    /// Parameters that can be customized
    pub parameters: Vec<WorkflowParameter>,
    /// When the workflow was created
    pub created_at: SystemTime,
    /// When the workflow was last modified
    pub modified_at: SystemTime,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Whether this workflow is a favorite
    pub favorite: bool,
    /// Usage count
    pub usage_count: u32,
    /// Last used timestamp
    pub last_used: Option<SystemTime>,
    /// Source (user-created, built-in, imported)
    pub source: WorkflowSource,
}

/// Workflow categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowCategory {
    /// Development workflows
    Development,
    /// Deployment workflows
    Deployment,
    /// Testing workflows
    Testing,
    /// Code quality workflows
    CodeQuality,
    /// Git workflows
    Git,
    /// Database workflows
    Database,
    /// Custom category
    Custom(String),
}

impl WorkflowCategory {
    /// Get icon for display
    pub fn icon(&self) -> &str {
        match self {
            WorkflowCategory::Development => "💻",
            WorkflowCategory::Deployment => "🚀",
            WorkflowCategory::Testing => "🧪",
            WorkflowCategory::CodeQuality => "✨",
            WorkflowCategory::Git => "🔀",
            WorkflowCategory::Database => "🗄️",
            WorkflowCategory::Custom(_) => "📁",
        }
    }
}

/// Workflow source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowSource {
    /// Built-in workflow
    BuiltIn,
    /// User-created workflow
    UserCreated,
    /// Imported from file
    Imported,
    /// Downloaded from community
    Community,
}

impl Workflow {
    /// Create a new workflow
    pub fn new(name: String, description: String, category: WorkflowCategory) -> Self {
        let now = SystemTime::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            category,
            commands: Vec::new(),
            parameters: Vec::new(),
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            favorite: false,
            usage_count: 0,
            last_used: None,
            source: WorkflowSource::UserCreated,
        }
    }

    /// Add a command to the workflow
    pub fn add_command(&mut self, command: String) {
        self.commands.push(command);
        self.modified_at = SystemTime::now();
    }

    /// Add a parameter to the workflow
    pub fn add_parameter(&mut self, parameter: WorkflowParameter) {
        self.parameters.push(parameter);
        self.modified_at = SystemTime::now();
    }

    /// Render commands with parameters applied
    pub fn render_commands(&self, params: &HashMap<String, String>) -> Vec<String> {
        self.commands
            .iter()
            .map(|cmd| {
                let mut rendered = cmd.clone();
                for param in &self.parameters {
                    let placeholder = format!("{{{{{}}}}}", param.name);
                    let value = params
                        .get(&param.name)
                        .or(param.default_value.as_ref())
                        .cloned()
                        .unwrap_or_default();
                    rendered = rendered.replace(&placeholder, &value);
                }
                rendered
            })
            .collect()
    }

    /// Validate parameters against workflow definition
    pub fn validate_parameters(&self, params: &HashMap<String, String>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for param in &self.parameters {
            if param.required && !params.contains_key(&param.name) && param.default_value.is_none()
            {
                errors.push(format!("Missing required parameter: {}", param.name));
            }

            if let Some(value) = params.get(&param.name) {
                match param.param_type {
                    ParameterType::Number if value.parse::<f64>().is_err() => {
                        errors.push(format!("Parameter '{}' must be a number", param.name));
                    }
                    ParameterType::Boolean
                        if value != "true" && value != "false" && value != "1" && value != "0" =>
                    {
                        errors.push(format!("Parameter '{}' must be a boolean", param.name));
                    }
                    ParameterType::Choice => {
                        if let Some(ref possible) = param.possible_values {
                            if !possible.contains(value) {
                                errors.push(format!(
                                    "Parameter '{}' must be one of: {}",
                                    param.name,
                                    possible.join(", ")
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Mark workflow as used
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used = Some(SystemTime::now());
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.favorite = !self.favorite;
    }

    /// Add tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove tag
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
}

/// Manages workflows collection
#[derive(Debug, Default)]
pub struct WorkflowManager {
    /// All workflows indexed by ID
    workflows: HashMap<String, Workflow>,
    /// Directory for saving workflows
    storage_dir: Option<PathBuf>,
}

impl WorkflowManager {
    /// Create a new workflow manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set storage directory
    pub fn with_storage(mut self, dir: PathBuf) -> Self {
        self.storage_dir = Some(dir);
        self
    }

    /// Create a new workflow
    pub fn create_workflow(
        &mut self,
        name: String,
        description: String,
        category: WorkflowCategory,
    ) -> &Workflow {
        let workflow = Workflow::new(name, description, category);
        let id = workflow.id.clone();
        self.workflows.insert(id.clone(), workflow);
        self.workflows.get(&id).unwrap()
    }

    /// Get a workflow by ID
    pub fn get_workflow(&self, id: &str) -> Option<&Workflow> {
        self.workflows.get(id)
    }

    /// Get a mutable reference to a workflow
    pub fn get_workflow_mut(&mut self, id: &str) -> Option<&mut Workflow> {
        self.workflows.get_mut(id)
    }

    /// List all workflows (optionally filtered by category)
    pub fn list_workflows(&self, category: Option<&WorkflowCategory>) -> Vec<&Workflow> {
        self.workflows
            .values()
            .filter(|w| {
                if let Some(c) = category {
                    w.category == *c
                } else {
                    true
                }
            })
            .collect()
    }

    /// Delete a workflow
    pub fn delete_workflow(&mut self, id: &str) -> bool {
        self.workflows.remove(id).is_some()
    }

    /// Search workflows
    pub fn search(&self, query: &str) -> Vec<&Workflow> {
        let query_lower = query.to_lowercase();
        self.workflows
            .values()
            .filter(|w| {
                w.name.to_lowercase().contains(&query_lower)
                    || w.description.to_lowercase().contains(&query_lower)
                    || w.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get favorite workflows
    pub fn favorite_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().filter(|w| w.favorite).collect()
    }

    /// Get most used workflows
    pub fn most_used(&self, limit: usize) -> Vec<&Workflow> {
        let mut workflows: Vec<&Workflow> = self.workflows.values().collect();
        workflows.sort_by_key(|w| std::cmp::Reverse(w.usage_count));
        workflows.into_iter().take(limit).collect()
    }

    /// Get recently used workflows
    pub fn recently_used(&self, limit: usize) -> Vec<&Workflow> {
        let mut workflows: Vec<&Workflow> = self.workflows.values().collect();
        workflows.sort_by_key(|w| w.last_used);
        workflows.into_iter().take(limit).collect()
    }

    /// Get workflow count
    pub fn workflow_count(&self) -> usize {
        self.workflows.len()
    }

    /// Save all workflows to disk
    pub fn save_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(dir) = &self.storage_dir {
            std::fs::create_dir_all(dir)?;
            let json = serde_json::to_string_pretty(&self.workflows)?;
            std::fs::write(dir.join("workflows.json"), json)?;
        }
        Ok(())
    }

    /// Load all workflows from disk
    pub fn load_all(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(dir) = &self.storage_dir {
            let path = dir.join("workflows.json");
            if path.exists() {
                let json = std::fs::read_to_string(path)?;
                self.workflows = serde_json::from_str(&json)?;
            }
        }
        Ok(())
    }

    /// Import workflow from JSON string
    pub fn import_workflow(
        &mut self,
        json: &str,
    ) -> Result<&Workflow, Box<dyn std::error::Error + Send + Sync>> {
        let mut workflow: Workflow = serde_json::from_str(json)?;
        workflow.source = WorkflowSource::Imported;
        let id = workflow.id.clone();
        self.workflows.insert(id.clone(), workflow);
        Ok(self.workflows.get(&id).unwrap())
    }

    /// Export workflow to JSON string
    pub fn export_workflow(
        &self,
        id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(workflow) = self.workflows.get(id) {
            Ok(serde_json::to_string_pretty(workflow)?)
        } else {
            Err("Workflow not found".into())
        }
    }

    /// Create built-in development workflows
    pub fn create_built_in_workflows(&mut self) {
        // Rust build workflow
        let mut rust_build = Workflow::new(
            "Rust Build".to_string(),
            "Build and test Rust project".to_string(),
            WorkflowCategory::Development,
        );
        rust_build.add_command("cargo build --release".to_string());
        rust_build.add_command("cargo test".to_string());
        rust_build.add_command("cargo clippy -- -D warnings".to_string());
        rust_build.source = WorkflowSource::BuiltIn;
        rust_build.add_tag("rust".to_string());
        rust_build.add_tag("build".to_string());
        let id = rust_build.id.clone();
        self.workflows.insert(id, rust_build);

        // Git workflow
        let mut git_workflow = Workflow::new(
            "Git Feature Branch".to_string(),
            "Create and switch to a new feature branch".to_string(),
            WorkflowCategory::Git,
        );
        git_workflow.add_command("git checkout -b feature/{{branch_name}}".to_string());
        git_workflow.add_parameter(WorkflowParameter {
            name: "branch_name".to_string(),
            description: "Name of the feature branch".to_string(),
            default_value: None,
            required: true,
            param_type: ParameterType::String,
            possible_values: None,
        });
        git_workflow.source = WorkflowSource::BuiltIn;
        git_workflow.add_tag("git".to_string());
        let id = git_workflow.id.clone();
        self.workflows.insert(id, git_workflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
            WorkflowCategory::Development,
        );
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.category, WorkflowCategory::Development);
        assert!(workflow.commands.is_empty());
    }

    #[test]
    fn test_workflow_parameters() {
        let mut workflow = Workflow::new(
            "Test".to_string(),
            "Test".to_string(),
            WorkflowCategory::Development,
        );

        workflow.add_parameter(WorkflowParameter {
            name: "env".to_string(),
            description: "Environment".to_string(),
            default_value: Some("dev".to_string()),
            required: false,
            param_type: ParameterType::String,
            possible_values: None,
        });

        workflow.add_command("echo {{env}}".to_string());

        let mut params = HashMap::new();
        params.insert("env".to_string(), "prod".to_string());

        let rendered = workflow.render_commands(&params);
        assert_eq!(rendered[0], "echo prod");
    }

    #[test]
    fn test_workflow_manager() {
        let mut manager = WorkflowManager::new();
        let workflow_id = {
            let workflow = manager.create_workflow(
                "Test".to_string(),
                "Test workflow".to_string(),
                WorkflowCategory::Development,
            );
            workflow.id.clone()
        };
        assert_eq!(manager.workflow_count(), 1);
        assert!(manager.get_workflow(&workflow_id).is_some());
    }
}
