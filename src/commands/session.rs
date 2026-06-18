use crate::cli::SessionCommands;
use crate::sessions::SessionManager;
use serde_json::Value;

pub async fn handle_session(
    cmd: SessionCommands,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let session_manager = SessionManager::new();

    match cmd {
        SessionCommands::List => {
            let sessions = session_manager.list_sessions();
            if sessions.is_empty() {
                println!("No sessions found.");
            } else {
                println!("Sessions:");
                for session in sessions {
                    println!(
                        "  {} - {} messages (created: {})",
                        session.id,
                        session.messages.len(),
                        session.timestamp.format("%Y-%m-%d %H:%M:%S")
                    );
                }
            }
        }
        SessionCommands::Show { id } => match session_manager.load_session(&id) {
            Ok(session) => {
                println!("Session: {}", session.id);
                println!("Created: {}", session.timestamp);
                println!("Messages: {}", session.messages.len());
                println!("\n--- Messages ---");
                for (i, msg) in session.messages.iter().enumerate() {
                    let role = msg
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let content = msg
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .chars()
                        .take(80)
                        .collect::<String>();
                    println!("{}. [{}] {}", i + 1, role, content);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        },
        SessionCommands::Delete { id } => match session_manager.delete_session(&id) {
            Ok(_) => println!("Session deleted."),
            Err(e) => eprintln!("Error: {}", e),
        },
        SessionCommands::Save { id, messages } => {
            let msgs: Vec<Value> = serde_json::from_str(&messages)
                .map_err(|e| format!("Invalid JSON: {}", e))
                .unwrap_or_default();

            match session_manager.save_session(&id, &msgs) {
                Ok(_) => println!("Session '{}' saved ({} messages).", id, msgs.len()),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        SessionCommands::Fork { id, name } => {
            match session_manager.fork_session(&id, name.as_deref()) {
                Ok(new_session) => {
                    println!("Forked session '{}' → '{}'", id, new_session.id);
                    println!("Timestamp: {}", new_session.timestamp);
                    println!("Messages copied: {}", new_session.messages.len());
                }
                Err(e) => {
                    eprintln!("Error forking session: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionCommands::Checkpoint { id, name } => {
            match session_manager.create_checkpoint(&id, name.as_deref()) {
                Ok(checkpoint) => {
                    println!(
                        "Checkpoint '{}' created for session '{}'",
                        checkpoint.name, id
                    );
                    println!("Timestamp: {}", checkpoint.timestamp);
                    println!("Messages: {}", checkpoint.messages.len());
                }
                Err(e) => {
                    eprintln!("Error creating checkpoint: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionCommands::CheckpointList { id } => {
            let checkpoints = session_manager.list_checkpoints(&id);
            if checkpoints.is_empty() {
                println!("No checkpoints found for session '{}'.", id);
            } else {
                println!("Checkpoints for session '{}':", id);
                for cp in checkpoints {
                    println!(
                        "  {} - {} messages (created: {})",
                        cp.name,
                        cp.messages.len(),
                        cp.timestamp.format("%Y-%m-%d %H:%M:%S")
                    );
                }
            }
        }
        SessionCommands::CheckpointRestore { id, name } => {
            match session_manager.restore_checkpoint(&id, &name) {
                Ok(session) => {
                    println!("Session '{}' restored from checkpoint '{}'", id, name);
                    println!("Timestamp: {}", session.timestamp);
                    println!("Messages: {}", session.messages.len());
                }
                Err(e) => {
                    eprintln!("Error restoring checkpoint: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionCommands::CheckpointDelete { id, name } => {
            match session_manager.delete_checkpoint(&id, &name) {
                Ok(_) => println!("Checkpoint '{}' deleted for session '{}'.", name, id),
                Err(e) => {
                    eprintln!("Error deleting checkpoint: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}
