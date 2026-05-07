use std::process::Command;

pub fn checkpoint() -> Result<String, String> {
    // Stage all changes
    let add_status = Command::new("git").arg("add").arg(".").status();

    match add_status {
        Ok(status) if status.success() => {
            // Commit
            let commit_output = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg("open_crust checkpoint")
                .output();

            match commit_output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if output.status.success() {
                        Ok("Checkpoint created.".to_string())
                    } else if stdout.contains("nothing to commit")
                        || stdout.contains("working tree clean")
                    {
                        Ok("No changes to checkpoint.".to_string())
                    } else {
                        Err(format!(
                            "Commit failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ))
                    }
                }
                Err(e) => Err(format!("Failed to execute git commit: {}", e)),
            }
        }
        Ok(_) => Err("git add failed".to_string()),
        Err(e) => Err(format!("Failed to execute git add: {}", e)),
    }
}

pub fn undo() -> Result<String, String> {
    let output = Command::new("git")
        .arg("reset")
        .arg("--hard")
        .arg("HEAD~1")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            Ok("Successfully undid the last checkpoint.".to_string())
        }
        Ok(out) => Err(format!(
            "Failed to undo: {}",
            String::from_utf8_lossy(&out.stderr)
        )),
        Err(e) => Err(format!("Error executing git reset: {}", e)),
    }
}

pub fn redo() -> Result<String, String> {
    Err("Redo is not yet implemented. You can manually inspect `git reflog`.".to_string())
}
