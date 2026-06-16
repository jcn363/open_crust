use crate::app::{App, ChangeStatus, Message, Mode, ProposedChange};
use tokio::sync::mpsc;

/// Handle incoming responses from the LLM task
pub fn handle_responses(app: &mut App, response_rx: &mut mpsc::Receiver<String>) {
    while let Ok(response) = response_rx.try_recv() {
        if response.contains("[APPROVAL_REQUIRED]") {
            app.waiting_for_approval = true;
        } else if response.starts_with("[DIFF_REQUIRED]") {
            let parts: Vec<&str> = response
                .strip_prefix("[DIFF_REQUIRED]")
                .map(|s| s.splitn(3, '|').collect())
                .unwrap_or_default();
            if parts.len() == 3 {
                app.proposed_changes.push(ProposedChange {
                    path: parts[0].to_string(),
                    original: parts[1].to_string(),
                    proposed: parts[2].to_string(),
                    status: ChangeStatus::Pending,
                });
                app.mode = Mode::Review;
            }
        }

        let tab_idx = app.active_tab.min(app.tabs.len().saturating_sub(1));
        let tab = &mut app.tabs[tab_idx];
        if let Some(last) = tab.messages.last()
            && (last.content == "opencrust: Thinking..."
                || last.content.starts_with("opencrust: Executing tool"))
        {
            tab.messages.pop();
        }
        tab.messages.push(Message::new(response));
        // Auto-scroll to bottom on new messages
        app.message_scroll = 0;
        app.mark_dirty();
    }
}
