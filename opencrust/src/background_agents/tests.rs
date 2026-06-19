use super::*;

#[test]
fn test_agent_creation() {
    let agent = BackgroundAgent::new(
        "test-agent".into(),
        "do something".into(),
        "ollama".into(),
        "llama3".into(),
    );
    assert_eq!(agent.name, "test-agent");
    assert_eq!(agent.status, AgentStatus::Pending);
    assert_eq!(agent.progress, 0);
    assert!(agent.log.is_empty());
}

#[test]
fn test_log_ring_buffer() {
    let mut agent = BackgroundAgent::new("t".into(), "p".into(), "o".into(), "m".into());
    for i in 0..300 {
        agent.log_line(format!("line {}", i));
    }
    assert_eq!(agent.log.len(), 256);
    assert!(agent.log[0].contains("line 44")); // 300-256 = 44
    assert!(agent.log[255].contains("line 299"));
}

#[test]
fn test_cancel_checks_status() {
    let agent = BackgroundAgent::new("t".into(), "p".into(), "o".into(), "m".into());
    assert_eq!(agent.status, AgentStatus::Pending);
    // Pending can be cancelled — we test via manager logic
}

#[test]
fn test_stats_default() {
    let stats = AgentStats::default();
    assert_eq!(stats.total, 0);
}

#[test]
fn test_agent_display_pending() {
    let agent = BackgroundAgent::new("x".into(), "p".into(), "o".into(), "m".into());
    let s = format!("{}", agent);
    assert!(s.contains("PENDING"));
}

#[test]
fn test_manager_empty_on_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mgr = BackgroundAgentManager::new();
    let agents = rt.block_on(async { mgr.list().await });
    assert!(agents.is_empty());
}

#[test]
fn test_manager_stats_empty() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mgr = BackgroundAgentManager::new();
    let stats = rt.block_on(async { mgr.stats().await });
    assert_eq!(stats.total, 0);
}
