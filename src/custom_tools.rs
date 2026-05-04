use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CustomTool {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub args: Vec<String>,
}

pub struct CustomToolManager {
    pub tools: HashMap<String, CustomTool>,
}

impl CustomToolManager {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn discover(&mut self) {
        let paths = vec![
            PathBuf::from(".opencrust/tools"),
            dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("open_crust/tools"),
        ];

        for path in paths {
            if path.exists() && path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() {
                            if let Some(tool) = self.parse_tool(&p) {
                                self.tools.insert(tool.name.clone(), tool);
                            }
                        }
                    }
                }
            }
        }
    }

    fn parse_tool(&self, path: &Path) -> Option<CustomTool> {
        let content = fs::read_to_string(path).ok()?;
        let mut name = None;
        let mut description = String::new();
        let mut args = Vec::new();

        for line in content.lines() {
            if line.starts_with("# name:") {
                name = Some(line.trim_start_matches("# name:").trim().to_string());
            } else if line.starts_with("# description:") {
                description = line.trim_start_matches("# description:").trim().to_string();
            } else if line.starts_with("# args:") {
                args = line.trim_start_matches("# args:")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            }
        }

        let name = name.or_else(|| {
            path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
        })?;

        Some(CustomTool {
            name,
            description,
            path: path.to_path_buf(),
            args,
        })
    }

    pub fn get_tools_schema(&self) -> Vec<Value> {
        self.tools.values().map(|tool| {
            let mut properties = json!({});
            let mut required = Vec::new();

            for arg in &tool.args {
                properties[arg] = json!({
                    "type": "string"
                });
                required.push(arg);
            }

            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": {
                        "type": "object",
                        "properties": properties,
                        "required": required
                    }
                }
            })
        }).collect()
    }

    pub async fn call_tool(&self, name: &str, args: &Value) -> Result<String, String> {
        let tool = self.tools.get(name).ok_or_else(|| format!("Tool '{}' not found", name))?;
        
        let mut command = Command::new(&tool.path);
        for arg_name in &tool.args {
            if let Some(val) = args.get(arg_name).and_then(|v| v.as_str()) {
                command.arg(val);
            } else {
                command.arg(""); // or error?
            }
        }

        let output = command.output().map_err(|e| format!("Failed to execute tool: {}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}
