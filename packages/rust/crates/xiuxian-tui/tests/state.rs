//! Integration tests for `xiuxian_tui::state`.

use xiuxian_tui::state::{AppState, ExecutionState, LogWindow, TaskItem, TaskStatus};

fn must_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(inner) => inner,
        None => panic!("{context}"),
    }
}

#[test]
fn test_task_item_creation() {
    let task = TaskItem::new(
        "t1".to_string(),
        "Test task".to_string(),
        "echo test".to_string(),
    );
    assert_eq!(task.id, "t1");
    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.status_symbol(), "○");
}

#[test]
fn test_task_status_colors() {
    let mut pending = TaskItem::new("t1".to_string(), "Test".to_string(), "cmd".to_string());
    let mut running = TaskItem::new("t2".to_string(), "Test".to_string(), "cmd".to_string());
    let mut success = TaskItem::new("t3".to_string(), "Test".to_string(), "cmd".to_string());

    pending.status = TaskStatus::Pending;
    running.status = TaskStatus::Running;
    success.status = TaskStatus::Success;

    assert_ne!(pending.status_color(), running.status_color());
    assert_ne!(running.status_color(), success.status_color());
}

#[test]
fn test_execution_state() {
    let mut state = ExecutionState::new();
    assert!(state.tasks.is_empty());

    state.add_task(TaskItem::new(
        "t1".to_string(),
        "Task 1".to_string(),
        "cmd1".to_string(),
    ));
    state.add_task(TaskItem::new(
        "t2".to_string(),
        "Task 2".to_string(),
        "cmd2".to_string(),
    ));

    assert_eq!(state.tasks.len(), 2);
    assert!(state.find_task("t1").is_some());
    assert!(state.find_task("unknown").is_none());

    state.update_task_status("t1", TaskStatus::Running);
    let t1 = must_some(state.find_task("t1"), "task t1 should exist");
    assert_eq!(t1.status, TaskStatus::Running);
}

#[test]
fn test_log_window_bounded() {
    let mut window = LogWindow::new(5);
    for i in 0..10 {
        window.add_line("info", &format!("Line {i}"), "");
    }
    assert_eq!(window.len(), 5);
    assert!(window.get_lines_owned()[0].contains("Line 5"));
}

#[test]
fn test_app_state_creation() {
    let state = AppState::new("Test App".to_string());
    assert_eq!(state.title(), "Test App");
    assert!(!state.should_quit());
    assert!(state.app().is_some());
}

#[test]
fn test_app_state_add_result() {
    let mut state = AppState::new("Test".to_string());
    state.add_result("Panel 1", "Content 1");

    let app = must_some(state.app(), "app should be initialized");
    assert_eq!(app.panels().len(), 1);
}

#[test]
fn test_app_state_quit() {
    let mut state = AppState::new("Test".to_string());
    assert!(!state.should_quit());

    state.quit();
    assert!(state.should_quit());
}
