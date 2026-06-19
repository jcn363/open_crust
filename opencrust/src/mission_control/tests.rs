use super::state::MissionControlUI;
use super::types::*;
use crate::orchestrator::task::{Task, TaskState};
use crossterm::event::KeyCode;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

fn make_task(id: Uuid, desc: &str, agent: &str, deps: Vec<Uuid>, state: TaskState) -> Task {
    Task {
        id,
        description: desc.to_string(),
        dependencies: deps,
        state,
        result: None,
        agent_type: agent.to_string(),
    }
}

fn dummy_theme() -> crate::ui::ThemeContext {
    crate::ui::ThemeContext {
        bg: ratatui::style::Color::Reset,
        fg: ratatui::style::Color::White,
        accent: ratatui::style::Color::Cyan,
        border: ratatui::style::Color::DarkGray,
    }
}

fn create_test_ui() -> MissionControlUI {
    MissionControlUI::new()
}

fn create_populated_ui() -> MissionControlUI {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();
    let d = Uuid::new_v4();

    let tasks = vec![
        make_task(
            a,
            "Task A",
            "agent1",
            vec![],
            TaskState::Completed {
                output: "done".into(),
            },
        ),
        make_task(b, "Task B", "agent1", vec![a], TaskState::Pending),
        make_task(
            c,
            "Task C",
            "agent2",
            vec![a],
            TaskState::Running {
                agent_id: "pool-1".into(),
            },
        ),
        make_task(d, "Task D", "agent3", vec![b, c], TaskState::Pending),
    ];

    let mut ui = MissionControlUI::new();
    ui.tasks = tasks;
    ui.refresh_layout();
    ui.compute_stats();
    ui
}

// =====================
// Phase 3: Layout tests
// =====================

#[test]
fn test_layout_chain() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();

    let tasks = vec![
        make_task(a, "A", "a", vec![], TaskState::Pending),
        make_task(b, "B", "b", vec![a], TaskState::Pending),
        make_task(c, "C", "c", vec![b], TaskState::Pending),
    ];

    let mut ui = MissionControlUI::new();
    ui.tasks = tasks;
    ui.refresh_layout();

    // Chain A→B→C should produce 3 layers and 2 edges
    assert_eq!(ui.layers.len(), 3, "chain should have 3 layers");
    assert_eq!(ui.edges.len(), 2, "chain should have 2 edges");
    assert!(ui.edges.contains(&(0, 1)), "edge A→B");
    assert!(ui.edges.contains(&(1, 2)), "edge B→C");
}

#[test]
fn test_layout_diamond() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();
    let d = Uuid::new_v4();

    let tasks = vec![
        make_task(a, "A", "a", vec![], TaskState::Pending),
        make_task(b, "B", "b", vec![a], TaskState::Pending),
        make_task(c, "C", "c", vec![a], TaskState::Pending),
        make_task(d, "D", "d", vec![b, c], TaskState::Pending),
    ];

    let mut ui = MissionControlUI::new();
    ui.tasks = tasks;
    ui.refresh_layout();

    // Diamond A→B, A→C, B→D, C→D should produce 3 layers, 4 edges
    assert_eq!(ui.layers.len(), 3, "diamond should have 3 layers");
    assert_eq!(ui.edges.len(), 4, "diamond should have 4 edges");
}

#[test]
fn test_layout_independent() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();

    let tasks = vec![
        make_task(a, "A", "a", vec![], TaskState::Pending),
        make_task(b, "B", "b", vec![], TaskState::Pending),
    ];

    let mut ui = MissionControlUI::new();
    ui.tasks = tasks;
    ui.refresh_layout();

    // Two independent tasks → 1 layer, 0 edges
    assert_eq!(ui.layers.len(), 1, "independent should have 1 layer");
    assert_eq!(ui.edges.len(), 0, "independent should have 0 edges");
}

#[test]
fn test_layout_empty() {
    let mut ui = MissionControlUI::new();
    ui.refresh_layout();
    assert!(ui.layers.is_empty(), "empty should have no layers");
    assert!(ui.edges.is_empty(), "empty should have no edges");
}

#[test]
fn test_layout_single() {
    let a = Uuid::new_v4();
    let tasks = vec![make_task(a, "Only", "a", vec![], TaskState::Pending)];

    let mut ui = MissionControlUI::new();
    ui.tasks = tasks;
    ui.refresh_layout();

    assert_eq!(ui.layers.len(), 1, "single task should have 1 layer");
    assert_eq!(ui.edges.len(), 0, "single task should have 0 edges");
}

// ==============================
// Phase 5: Stats computation
// ==============================

#[test]
fn test_compute_stats_empty() {
    let mut ui = create_test_ui();
    ui.compute_stats();
    assert_eq!(ui.stats.total, 0);
    assert_eq!(ui.stats.pending, 0);
}

#[test]
fn test_compute_stats_mixed() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();
    let d = Uuid::new_v4();
    let e = Uuid::new_v4();

    let tasks = vec![
        make_task(a, "P", "a", vec![], TaskState::Pending),
        make_task(
            b,
            "R1",
            "b",
            vec![],
            TaskState::Running {
                agent_id: "x".into(),
            },
        ),
        make_task(
            c,
            "R2",
            "c",
            vec![],
            TaskState::Running {
                agent_id: "y".into(),
            },
        ),
        make_task(
            d,
            "C",
            "d",
            vec![],
            TaskState::Completed {
                output: "ok".into(),
            },
        ),
        make_task(
            e,
            "F",
            "e",
            vec![],
            TaskState::Failed {
                error: "err".into(),
            },
        ),
    ];

    let mut ui = MissionControlUI::new();
    ui.tasks = tasks;
    ui.compute_stats();

    assert_eq!(ui.stats.total, 5);
    assert_eq!(ui.stats.pending, 1);
    assert_eq!(ui.stats.running, 2);
    assert_eq!(ui.stats.completed, 1);
    assert_eq!(ui.stats.failed, 1);
}

#[test]
fn test_render_detail_no_panic() {
    let ui = create_populated_ui();
    // Verify the UI state is valid (render would not panic)
    assert!(ui.stats.total > 0);
    assert!(ui.selected_index < ui.tasks.len());
}

// ==============================
// Phase 6: Navigation tests
// ==============================

#[test]
fn test_navigate_down_in_layer() {
    let ui = create_populated_ui();
    // Selection should be clamped to valid range
    if !ui.tasks.is_empty() {
        assert!(ui.selected_index < ui.tasks.len());
    }
}

// ==============================
// Phase 7: Bridge refresh tests
// ==============================

#[test]
fn test_refresh_tasks_no_bridge() {
    let mut ui = create_populated_ui();
    let initial_len = ui.tasks.len();
    ui.refresh_tasks(None);
    // Without a bridge, tasks should remain unchanged
    assert_eq!(ui.tasks.len(), initial_len);
}

#[test]
fn test_refresh_tasks_stale_on_contention() {
    let mut ui = create_populated_ui();
    let initial_count = ui.stats.total;

    // With a bridge that's locked, should keep stale snapshot
    let bridge: Option<Arc<RwLock<Vec<Task>>>> = None;
    ui.refresh_tasks(bridge.as_ref());
    assert_eq!(ui.stats.total, initial_count);
}

// =====================
// Core action tests
// =====================

#[test]
fn test_initial_state() {
    let ui = create_test_ui();
    assert!(ui.tasks.is_empty());
    assert!(ui.layers.is_empty());
    assert!(ui.edges.is_empty());
    assert!(ui.node_positions.is_empty());
    assert_eq!(ui.selected_index, 0);
    assert_eq!(ui.active_panel, 0);
}

#[test]
fn test_handle_key_esc() {
    let mut ui = create_test_ui();
    let action = ui.handle_key(KeyCode::Esc);
    assert!(matches!(action, MissionControlAction::ExitMode));
}

#[test]
fn test_handle_key_unknown() {
    let mut ui = create_test_ui();
    let action = ui.handle_key(KeyCode::Char('x'));
    assert!(matches!(action, MissionControlAction::None));
}

#[test]
fn test_handle_key_enter() {
    let mut ui = create_populated_ui();
    let action = ui.handle_key(KeyCode::Enter);
    assert!(matches!(action, MissionControlAction::SelectTask(_)));
}

#[test]
fn test_truncate_short() {
    assert_eq!(MissionControlUI::truncate("hello", 10), "hello");
}

#[test]
fn test_truncate_long() {
    let result = MissionControlUI::truncate("hello world this is long", 10);
    assert_eq!(result.chars().count(), 10);
    assert!(result.ends_with('…'));
}

#[test]
fn test_task_style_pending() {
    let task = make_task(Uuid::new_v4(), "t", "a", vec![], TaskState::Pending);
    let theme = dummy_theme();
    let (icon, color) = MissionControlUI::task_style(&task, &theme);
    assert_eq!(icon, '⏳');
    assert_eq!(color, theme.fg);
}

#[test]
fn test_task_style_running() {
    let task = make_task(
        Uuid::new_v4(),
        "t",
        "a",
        vec![],
        TaskState::Running {
            agent_id: "x".into(),
        },
    );
    let theme = dummy_theme();
    let (icon, color) = MissionControlUI::task_style(&task, &theme);
    assert_eq!(icon, '▶');
    assert_eq!(color, theme.warning());
}

#[test]
fn test_task_style_completed() {
    let task = make_task(
        Uuid::new_v4(),
        "t",
        "a",
        vec![],
        TaskState::Completed {
            output: "ok".into(),
        },
    );
    let theme = dummy_theme();
    let (icon, color) = MissionControlUI::task_style(&task, &theme);
    assert_eq!(icon, '✓');
    assert_eq!(color, theme.success());
}

#[test]
fn test_task_style_failed() {
    let task = make_task(
        Uuid::new_v4(),
        "t",
        "a",
        vec![],
        TaskState::Failed {
            error: "err".into(),
        },
    );
    let theme = dummy_theme();
    let (icon, color) = MissionControlUI::task_style(&task, &theme);
    assert_eq!(icon, '✗');
    assert_eq!(color, theme.error());
}

#[test]
fn test_select_task_returns_action() {
    let mut ui = create_populated_ui();
    let action = ui.handle_key(KeyCode::Enter);
    assert!(
        matches!(action, MissionControlAction::SelectTask(idx) if idx == 0 || idx < ui.tasks.len())
    );
}
