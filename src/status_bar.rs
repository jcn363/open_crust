use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StatusBarState {
    pub provider: String,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

impl StatusBarState {
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            prompt_tokens: 0,
            completion_tokens: 0,
        }
    }
}

pub type SharedStatusBarState = Arc<RwLock<StatusBarState>>;

// Helper functions to update the state
#[allow(dead_code)]
pub async fn set_provider_model(state: &SharedStatusBarState, provider: &str, model: &str) {
    let mut w = state.write().await;
    w.provider = provider.to_string();
    w.model = model.to_string();
}

#[allow(dead_code)]
pub async fn update_usage(state: &SharedStatusBarState, prompt: u64, completion: u64) {
    let mut w = state.write().await;
    w.prompt_tokens = prompt;
    w.completion_tokens = completion;
}
