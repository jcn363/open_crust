use super::*;
use std::fs;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Serializes tests that write to plan.md (shared file between parallel tests)
static PLAN_MD_LOCK: Mutex<()> = Mutex::new(());

static PLAN_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_title() -> String {
    let n = PLAN_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test_plan_{}", n)
}

/// Helper: runs a closure inside a mutex lock (serializes plan.md access).
/// Recovers from poisoned mutex so one test failure doesn't cascade.
fn with_plan_md<F>(f: F)
where
    F: FnOnce(),
{
    let _lock = PLAN_MD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = fs::remove_file("plan.md");
    f();
    let _ = fs::remove_file("plan.md");
}

/// Runs a planner test that calls create_plan inside the serialized lock.
/// Every test that calls create_plan must go through here since it writes to plan.md.
fn run_test<F>(f: F)
where
    F: FnOnce(&mut Planner),
{
    with_plan_md(|| {
        let mut planner = Planner::new();
        f(&mut planner);
    });
}

#[test]
fn new_should_create_empty_planner() {
    let planner = Planner::new();
    assert!(planner.current_plan.is_none());
}

#[test]
fn create_plan_should_set_current_plan() {
    run_test(|planner| {
        let steps = vec!["Step 1".to_string(), "Step 2".to_string()];
        let title = unique_title();
        planner.create_plan(&title, steps.clone());

        let plan = planner.current_plan.as_ref().expect("plan should exist");
        assert_eq!(plan.title, title);
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].description, "Step 1");
        assert!(!plan.steps[0].completed);
        assert_eq!(plan.steps[1].description, "Step 2");
        assert!(!plan.steps[1].completed);
    });
}

#[test]
fn create_plan_should_write_plan_md() {
    run_test(|planner| {
        let steps = vec!["Alpha".to_string(), "Beta".to_string()];
        let title = unique_title();
        planner.create_plan(&title, steps);

        let content = fs::read_to_string("plan.md").expect("plan.md should exist");
        assert!(content.contains(&title));
        assert!(content.contains("- [ ] Alpha"));
        assert!(content.contains("- [ ] Beta"));
    });
}

#[test]
fn mark_step_complete_should_update_plan() {
    run_test(|planner| {
        let steps = vec!["First".to_string(), "Second".to_string()];
        let title = unique_title();
        planner.create_plan(&title, steps);

        let result = planner.mark_step_complete(0);
        assert!(result.contains("Step 0"));
        assert!(planner.current_plan.as_ref().unwrap().steps[0].completed);
        assert!(!planner.current_plan.as_ref().unwrap().steps[1].completed);
    });
}

#[test]
fn mark_step_complete_without_active_plan_should_error() {
    let mut planner = Planner::new();
    let result = planner.mark_step_complete(0);
    assert_eq!(result, "No active plan found.");
}

#[test]
fn mark_step_complete_out_of_range_should_error() {
    run_test(|planner| {
        let title = unique_title();
        planner.create_plan(&title, vec!["Only step".to_string()]);
        let result = planner.mark_step_complete(5);
        assert!(result.contains("out of range"));
    });
}

#[test]
fn create_plan_return_message_should_include_title_and_count() {
    run_test(|planner| {
        let result = planner.create_plan(
            &unique_title(),
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
        );
        assert!(result.contains("3 steps"));
    });
}

#[test]
fn mark_step_complete_should_update_plan_md() {
    run_test(|planner| {
        let steps = vec!["First".to_string(), "Second".to_string()];
        let title = unique_title();
        planner.create_plan(&title, steps);
        planner.mark_step_complete(1);

        let content = fs::read_to_string("plan.md").expect("plan.md should exist");
        assert!(content.contains("- [ ] First"));
        assert!(content.contains("- [x] Second"));
    });
}

#[test]
fn multiple_plans_should_replace_previous() {
    run_test(|planner| {
        planner.create_plan("first", vec!["Step A".to_string()]);
        planner.create_plan("second", vec!["Step B".to_string()]);

        let plan = planner.current_plan.as_ref().expect("plan should exist");
        assert_eq!(plan.title, "second");
        assert_eq!(plan.steps[0].description, "Step B");
    });
}
