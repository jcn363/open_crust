use super::*;

#[test]
fn test_new_manager_is_empty() {
    let manager = CustomCommandManager::new();
    assert!(manager.commands.is_empty());
}

#[test]
fn test_has_command_returns_false_for_unknown() {
    let manager = CustomCommandManager::new();
    assert!(!manager.has_command("nonexistent"));
}

#[test]
fn test_execute_unknown_command() {
    let manager = CustomCommandManager::new();
    let result = manager.execute_command("nonexistent", "");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"));
}

#[test]
fn test_list_commands_empty() {
    let manager = CustomCommandManager::new();
    let list = manager.list_commands();
    assert!(list.is_empty());
}
