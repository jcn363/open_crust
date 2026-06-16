use std::time::{Duration, Instant};

/// Frame rate limiter for smooth UI rendering (~60 FPS)
pub struct FrameLimiter {
    last_frame: Instant,
    target_frame_duration: Duration,
}

impl FrameLimiter {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            target_frame_duration: Duration::from_millis(16), // ~60 FPS
        }
    }

    /// Wait until the target frame duration has elapsed
    pub async fn wait_for_next_frame(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame);
        if elapsed < self.target_frame_duration {
            tokio::time::sleep(self.target_frame_duration - elapsed).await;
        }
        self.last_frame = Instant::now();
    }
}

impl Default for FrameLimiter {
    fn default() -> Self {
        Self::new()
    }
}