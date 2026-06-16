use crate::app::App;
use tokio::sync::mpsc;

/// Handle background task notifications
pub fn handle_background_tasks(
    app: &mut App,
    background_task_rx: &mut mpsc::Receiver<String>,
) {
    while let Ok(notification) = background_task_rx.try_recv() {
        app.handle_background_notification(&notification);
        app.mark_dirty();
    }
}

/// Spawn a background task with the given prompt
pub fn spawn_background_task(app: &mut App, prompt: String) {
    app.spawn_background_task(prompt);
}