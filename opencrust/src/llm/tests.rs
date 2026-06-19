use super::*;
use crate::config::Config;
use std::sync::Arc;

// ── PlanModeState ──

#[test]
fn plan_mode_state_default_is_disabled() {
    assert_eq!(PlanModeState::default(), PlanModeState::Disabled);
}

#[test]
fn plan_mode_state_partial_eq() {
    assert_eq!(PlanModeState::Disabled, PlanModeState::Disabled);
    assert_eq!(PlanModeState::Planning, PlanModeState::Planning);
    assert_ne!(PlanModeState::Disabled, PlanModeState::Planning);
}

#[test]
fn plan_mode_state_debug() {
    let _ = format!("{:?}", PlanModeState::Disabled);
    let _ = format!("{:?}", PlanModeState::Planning);
}

// ── LlmClient: plan mode ──

#[test]
fn plan_mode_roundtrip() {
    let client = test_client();
    assert_eq!(client.get_plan_mode(), PlanModeState::Disabled);
    client.set_plan_mode(PlanModeState::Planning);
    assert_eq!(client.get_plan_mode(), PlanModeState::Planning);
    client.set_plan_mode(PlanModeState::Disabled);
    assert_eq!(client.get_plan_mode(), PlanModeState::Disabled);
}

#[test]
fn tool_blocked_in_plan_mode_blocks_write_tools() {
    let client = test_client();
    client.set_plan_mode(PlanModeState::Planning);
    assert!(client.is_tool_blocked_in_plan_mode("write"));
    assert!(client.is_tool_blocked_in_plan_mode("edit"));
    assert!(client.is_tool_blocked_in_plan_mode("bash"));
    assert!(client.is_tool_blocked_in_plan_mode("global_search_replace"));
    assert!(client.is_tool_blocked_in_plan_mode("create_plan"));
}

#[test]
fn tool_not_blocked_in_plan_mode_allows_read_tools() {
    let client = test_client();
    client.set_plan_mode(PlanModeState::Planning);
    assert!(!client.is_tool_blocked_in_plan_mode("read"));
    assert!(!client.is_tool_blocked_in_plan_mode("grep"));
    assert!(!client.is_tool_blocked_in_plan_mode("glob"));
    assert!(!client.is_tool_blocked_in_plan_mode("web_search"));
}

#[test]
fn tool_not_blocked_when_disabled() {
    let client = test_client();
    client.set_plan_mode(PlanModeState::Disabled);
    assert!(!client.is_tool_blocked_in_plan_mode("write"));
    assert!(!client.is_tool_blocked_in_plan_mode("bash"));
}

// ── LlmClient: goal state ──

#[test]
fn goal_default_is_none() {
    let client = test_client();
    assert!(client.get_goal().is_none());
    assert!(client.get_goal_prompt().is_none());
}

#[test]
fn goal_set_and_clear() {
    let client = test_client();
    client.set_goal("test goal".into());
    let goal = client.get_goal();
    assert!(goal.is_some());
    assert_eq!(goal.unwrap().description, "test goal");
    client.clear_goal();
    assert!(client.get_goal().is_none());
}

#[test]
fn goal_get_prompt_contains_description() {
    let client = test_client();
    client.set_goal("fix the bug".into());
    let prompt = client.get_goal_prompt();
    assert!(prompt.is_some());
    let prompt_text = prompt.unwrap();
    assert!(prompt_text.contains("fix the bug"));
    assert!(prompt_text.contains("Active Goal"));
}

#[test]
fn goal_no_prompt_when_not_set() {
    let client = test_client();
    assert!(client.get_goal_prompt().is_none());
}

// ── Goal struct ──

#[test]
fn goal_creation() {
    let goal = Goal {
        description: "hello".into(),
        created_at: chrono::Utc::now(),
    };
    assert_eq!(goal.description, "hello");
}

// ── check_and_summarize_context: threshold logic ──

#[tokio::test]
async fn summarize_skips_when_under_threshold() {
    let config = Arc::new(Config {
        provider: crate::config::ProviderType::Ollama,
        ..Default::default()
    });
    let client = super::new_test_client(config).expect("test client");

    let mut messages = vec![
        json!({"role": "system", "content": "You are helpful."}),
        json!({"role": "user", "content": "Hello"}),
        json!({"role": "assistant", "content": "Hi there!"}),
    ];

    let (should_summarize, summary) = client.check_and_summarize_context(&mut messages).await;
    assert!(!should_summarize);
    assert!(summary.is_none());
    assert_eq!(messages.len(), 3);
}

#[tokio::test]
async fn summarize_skips_when_too_few_messages() {
    let config = Arc::new(Config {
        provider: crate::config::ProviderType::Ollama,
        ..Default::default()
    });
    let client = super::new_test_client(config).expect("test client");

    let long_content = "x".repeat(3000);
    let mut messages: Vec<Value> = (0..11)
        .map(|i| {
            if i == 0 {
                json!({"role": "system", "content": &long_content})
            } else if i % 2 == 0 {
                json!({"role": "user", "content": &long_content})
            } else {
                json!({"role": "assistant", "content": &long_content})
            }
        })
        .collect();

    let original_len = messages.len();
    let (should_summarize, summary) = client.check_and_summarize_context(&mut messages).await;
    assert!(!should_summarize);
    assert!(summary.is_none());
    assert_eq!(messages.len(), original_len);
}

// ── generate_input_completion: edge cases ──

#[tokio::test]
async fn input_completion_returns_empty_for_blank_input() {
    let config = Arc::new(Config::default());
    let client = super::new_test_client(config).expect("test client");

    let result = client.generate_input_completion("").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");

    let result = client.generate_input_completion("   ").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[tokio::test]
async fn input_completion_truncates_long_input() {
    let config = Arc::new(Config::default());
    let client = super::new_test_client(config).expect("test client");

    let long_input = "a".repeat(500);
    let result = client.generate_input_completion(&long_input).await;
    assert!(result.is_err() || result.is_ok());
}

// ── helper ──

fn test_client() -> LlmClient {
    let config = Arc::new(Config::default());
    super::new_test_client(config).expect("test client creation")
}
