use super::*;
use tokio::sync::mpsc;

// --- Mode enum ---

#[test]
fn mode_debug_and_clone() {
    let modes = [Mode::Normal, Mode::Insert, Mode::Review, Mode::Servers];
    for m in &modes {
        let _ = format!("{m:?}");
    }
}

// --- PlanMode enum ---

#[test]
fn plan_mode_default_is_disabled() {
    assert_eq!(PlanMode::default(), PlanMode::Disabled);
}

#[test]
fn plan_mode_partial_eq() {
    assert_eq!(PlanMode::Disabled, PlanMode::Disabled);
    assert_eq!(PlanMode::Planning, PlanMode::Planning);
    assert_ne!(PlanMode::Disabled, PlanMode::Planning);
}

// --- Message ---

#[test]
fn message_new_sets_content() {
    let msg = Message::new("hello".to_string());
    assert_eq!(msg.content, "hello");
}

#[test]
fn message_new_sets_timestamp() {
    let msg = Message::new("test".to_string());
    let now = chrono::Utc::now();
    let diff = now - msg.timestamp;
    assert!(diff.num_seconds() < 5, "timestamp should be recent");
}

// --- ChangeStatus ---

#[test]
fn change_status_variants() {
    assert_eq!(ChangeStatus::Pending, ChangeStatus::Pending);
    assert_eq!(ChangeStatus::Approved, ChangeStatus::Approved);
    assert_eq!(ChangeStatus::Denied, ChangeStatus::Denied);
    assert_ne!(ChangeStatus::Pending, ChangeStatus::Approved);
}

// --- TaskStatus ---

#[test]
fn task_status_variants() {
    assert_eq!(TaskStatus::Running, TaskStatus::Running);
    assert_eq!(TaskStatus::Completed, TaskStatus::Completed);
    assert_eq!(TaskStatus::Failed, TaskStatus::Failed);
}

// --- ProposedChange ---

#[test]
fn proposed_change_defaults_to_pending() {
    let pc = ProposedChange {
        path: "/tmp/f".into(),
        original: "a".into(),
        proposed: "b".into(),
        status: ChangeStatus::Pending,
    };
    assert_eq!(pc.status, ChangeStatus::Pending);
    assert_eq!(pc.path, "/tmp/f");
}

// --- BackgroundTask ---

#[test]
fn background_task_status_starts_running() {
    let task = BackgroundTask {
        id: "id-1".into(),
        prompt: "hello".into(),
        status: TaskStatus::Running,
        result: None,
        started_at: chrono::Utc::now(),
    };
    assert_eq!(task.status, TaskStatus::Running);
    assert!(task.result.is_none());
}

// --- App: mode transitions ---

#[test]
fn app_enter_insert_mode() {
    let mut app = app_for_test();
    app.mode = Mode::Normal;
    app.enter_insert_mode();
    assert_eq!(app.mode, Mode::Insert);
}

#[test]
fn app_enter_normal_mode() {
    let mut app = app_for_test();
    app.mode = Mode::Insert;
    app.enter_normal_mode();
    assert_eq!(app.mode, Mode::Normal);
}

// --- App: ghost text / prediction ---

#[test]
fn should_trigger_prediction_disabled() {
    let app = app_for_test();
    assert!(!app.should_trigger_prediction());
}

#[test]
fn should_trigger_prediction_no_last_time() {
    let mut app = app_for_test();
    app.input_prediction_enabled = true;
    assert!(!app.should_trigger_prediction());
}

#[test]
fn clear_ghost_text_clears_state() {
    let mut app = app_for_test();
    app.ghost_text = Some("test".into());
    app.last_input_time = Some(std::time::Instant::now());
    app.clear_ghost_text();
    assert!(app.ghost_text.is_none());
    assert!(app.last_input_time.is_none());
}

// --- App: history navigation ---

#[test]
fn history_up_empty_does_nothing() {
    let mut app = app_for_test();
    app.history_up();
    assert!(app.history_index.is_none());
}

#[test]
fn history_up_navigates_entries() {
    let mut app = app_for_test();
    app.history = vec!["first".into(), "second".into()];
    app.history_up();
    assert_eq!(app.history_index, Some(1));
    assert_eq!(app.input, "second");
}

#[test]
fn history_up_stays_at_zero() {
    let mut app = app_for_test();
    app.history = vec!["only".into()];
    app.history_index = Some(0);
    app.history_up();
    assert_eq!(app.history_index, Some(0));
}

#[test]
fn history_down_clears_at_end() {
    let mut app = app_for_test();
    app.history = vec!["a".into()];
    app.history_index = Some(0);
    app.history_down();
    assert!(app.history_index.is_none());
    assert!(app.input.is_empty());
}

#[test]
fn history_down_moves_forward() {
    let mut app = app_for_test();
    app.history = vec!["a".into(), "b".into()];
    app.history_index = Some(0);
    app.history_down();
    assert_eq!(app.history_index, Some(1));
    assert_eq!(app.input, "b");
}

#[test]
fn history_down_none_does_nothing() {
    let mut app = app_for_test();
    app.history_down();
    assert!(app.history_index.is_none());
}

// --- App: input manipulation ---

#[test]
fn handle_char_appends_to_input() {
    let mut app = app_for_test();
    app.handle_char('x');
    assert_eq!(app.input, "x");
}

#[test]
fn handle_backspace_removes_last_char() {
    let mut app = app_for_test();
    app.input = "ab".into();
    app.handle_backspace();
    assert_eq!(app.input, "a");
}

#[test]
fn handle_backspace_empty_does_nothing() {
    let mut app = app_for_test();
    app.handle_backspace();
    assert!(app.input.is_empty());
}

// --- App: vim cursor ---

#[test]
fn vim_cursor_left_decrements() {
    let mut app = app_for_test();
    app.vim_cursor_pos = 3;
    app.move_cursor_left();
    assert_eq!(app.vim_cursor_pos, 2);
}

#[test]
fn vim_cursor_left_stays_at_zero() {
    let mut app = app_for_test();
    app.vim_cursor_pos = 0;
    app.move_cursor_left();
    assert_eq!(app.vim_cursor_pos, 0);
}

#[test]
fn vim_cursor_right_increments() {
    let mut app = app_for_test();
    app.input = "abc".into();
    app.vim_cursor_pos = 1;
    app.move_cursor_right();
    assert_eq!(app.vim_cursor_pos, 2);
}

#[test]
fn vim_cursor_right_stays_at_len() {
    let mut app = app_for_test();
    app.input = "ab".into();
    app.vim_cursor_pos = 2;
    app.move_cursor_right();
    assert_eq!(app.vim_cursor_pos, 2);
}

#[test]
fn vim_next_word_skips_whitespace() {
    let mut app = app_for_test();
    app.input = "hello world".into();
    app.vim_cursor_pos = 0;
    app.move_to_next_word();
    assert_eq!(app.vim_cursor_pos, 6);
}

#[test]
fn vim_prev_word_goes_back() {
    let mut app = app_for_test();
    app.input = "hello world".into();
    app.vim_cursor_pos = 7;
    app.move_to_prev_word();
    assert_eq!(app.vim_cursor_pos, 6);
}

#[test]
fn vim_line_start() {
    let mut app = app_for_test();
    app.vim_cursor_pos = 5;
    app.move_to_line_start();
    assert_eq!(app.vim_cursor_pos, 0);
}

#[test]
fn vim_line_end() {
    let mut app = app_for_test();
    app.input = "hello".into();
    app.vim_cursor_pos = 0;
    app.move_to_line_end();
    assert_eq!(app.vim_cursor_pos, 5);
}

#[test]
fn vim_delete_line() {
    let mut app = app_for_test();
    app.input = "hello".into();
    app.vim_cursor_pos = 3;
    app.delete_line();
    assert!(app.input.is_empty());
    assert_eq!(app.vim_cursor_pos, 0);
}

// --- App: background notifications ---

#[test]
fn handle_bg_completion_updates_task() {
    let mut app = app_for_test();
    let task_id = "bg-001".to_string();
    app.background_tasks.push(BackgroundTask {
        id: task_id.clone(),
        prompt: "hello".into(),
        status: TaskStatus::Running,
        result: None,
        started_at: chrono::Utc::now(),
    });
    let result = app.handle_background_notification(&format!("[TASK_COMPLETE]{}::done", task_id));
    assert!(result.is_some());
    assert_eq!(app.background_tasks[0].status, TaskStatus::Completed);
    assert_eq!(app.background_tasks[0].result, Some("done".into()));
}

#[test]
fn handle_bg_failure_updates_task() {
    let mut app = app_for_test();
    let task_id = "bg-002".to_string();
    app.background_tasks.push(BackgroundTask {
        id: task_id.clone(),
        prompt: "hello".into(),
        status: TaskStatus::Running,
        result: None,
        started_at: chrono::Utc::now(),
    });
    let result =
        app.handle_background_notification(&format!("[TASK_FAILED]{}::error msg", task_id));
    assert!(result.is_some());
    assert_eq!(app.background_tasks[0].status, TaskStatus::Failed);
    assert_eq!(app.background_tasks[0].result, Some("error msg".into()));
}

#[test]
fn handle_bg_unknown_notification_returns_none() {
    let mut app = app_for_test();
    let result = app.handle_background_notification("some random text");
    assert!(result.is_none());
}

#[test]
fn handle_bg_malformed_notification_returns_none() {
    let mut app = app_for_test();
    let result = app.handle_background_notification("[TASK_COMPLETE]no_separator");
    assert!(result.is_none());
}

#[test]
fn submit_message_adds_to_tab_and_history() {
    let mut app = app_for_test();
    app.input = "hello".into();
    let initial_count = app.tabs[0].messages.len();
    app.submit_message();
    assert!(app.tabs[0].messages.len() > initial_count);
    assert!(
        app.tabs[0]
            .messages
            .last()
            .unwrap()
            .content
            .contains("hello")
    );
    assert!(app.history.contains(&"hello".into()));
    assert!(app.input.is_empty());
}

#[test]
fn submit_message_empty_does_nothing() {
    let mut app = app_for_test();
    app.history.clear();
    app.submit_message();
    assert!(app.history.is_empty());
    assert!(app.input.is_empty());
}

#[test]
fn submit_message_does_not_duplicate_history() {
    let mut app = app_for_test();
    app.history = vec!["same".into()];
    app.input = "same".into();
    app.submit_message();
    assert_eq!(app.history.len(), 1);
}

// --- App: tab management ---

#[test]
fn tabs_initialized_with_chat_and_tasks() {
    let app = app_for_test();
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.tabs[0].name, "Chat");
    assert_eq!(app.tabs[1].name, "Tasks");
    assert_eq!(app.active_tab, 0);
}

#[test]
fn app_default_mode_is_normal() {
    let app = app_for_test();
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn app_plan_mode_default_disabled() {
    let app = app_for_test();
    assert_eq!(app.plan_mode, PlanMode::Disabled);
}

// --- helpers ---

fn app_for_test() -> App {
    use std::sync::Arc;
    let (prompt_tx, _) = mpsc::channel(16);
    let (approval_tx, _) = mpsc::channel(16);
    let (bg_tx, _) = mpsc::channel(16);
    let config = Config::default();
    let config_arc = Arc::new(config.clone());
    let llm_client = crate::llm::new_test_client(config_arc).expect("test LlmClient creation");
    let mut app = App::new(config, prompt_tx, approval_tx, bg_tx, llm_client);
    app.history.clear();
    app.history_index = None;
    app
}
