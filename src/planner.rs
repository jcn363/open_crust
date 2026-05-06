use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plan {
    pub title: String,
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanStep {
    pub description: String,
    pub completed: bool,
}

pub struct Planner {
    pub current_plan: Option<Plan>,
}

impl Planner {
    pub fn new() -> Self {
        Self { current_plan: None }
    }

    pub fn create_plan(&mut self, title: &str, steps: Vec<String>) -> String {
        let plan = Plan {
            title: title.to_string(),
            steps: steps.into_iter().map(|s| PlanStep { description: s, completed: false }).collect(),
        };
        self.current_plan = Some(plan.clone());
        
        // Persist to plan.md for visibility
        let mut md = format!("# {}\n\n", plan.title);
        for step in &plan.steps {
            md.push_str(&format!("- [ ] {}\n", step.description));
        }
        let _ = fs::write("plan.md", md);
        
        format!("Plan '{}' created with {} steps. Progress tracked in plan.md.", title, plan.steps.len())
    }

    pub fn mark_step_complete(&mut self, index: usize) -> String {
        if let Some(plan) = &mut self.current_plan {
            if index < plan.steps.len() {
                plan.steps[index].completed = true;
                
                // Update plan.md
                let mut md = format!("# {}\n\n", plan.title);
                for step in plan.steps.iter() {
                    let mark = if step.completed { "x" } else { " " };
                    md.push_str(&format!("- [{}] {}\n", mark, step.description));
                }
                let _ = fs::write("plan.md", md);
                
                format!("Step {} marked as complete.", index)
            } else {
                format!("Step index {} out of range.", index)
            }
        } else {
            "No active plan found.".to_string()
        }
    }
}
