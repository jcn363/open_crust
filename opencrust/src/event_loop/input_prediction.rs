use crate::app::App;
use crate::llm::LlmClient;
use tokio::sync::mpsc;

/// Handle incoming prediction results
pub fn handle_prediction_results(
    app: &mut App,
    prediction_rx: &mut mpsc::Receiver<(String, String)>,
) {
    if let Ok((input_text, prediction)) = prediction_rx.try_recv()
        && input_text == app.input
    {
        app.ghost_text = Some(prediction);
        app.mark_dirty();
    }
}

/// Trigger input prediction if needed (after debounce)
pub fn trigger_prediction_if_needed(
    app: &mut App,
    llm_client: &LlmClient,
    prediction_tx: &mpsc::Sender<(String, String)>,
) {
    if app.should_trigger_prediction() && app.ghost_text.is_none() && !app.input.is_empty() {
        let llm_client_clone = llm_client.clone();
        let input = app.input.clone();
        let tx = prediction_tx.clone();
        tokio::spawn(async move {
            if let Ok(prediction) = llm_client_clone.generate_input_completion(&input).await {
                let _ = tx.send((input, prediction)).await;
            }
        });
        app.last_input_time = None; // Reset to prevent re-triggering
    }
}
