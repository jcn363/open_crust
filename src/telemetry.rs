use crate::stats::UsageStats;
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TelemetryExporter {
    pub usage_stats: Arc<Mutex<UsageStats>>,
}

impl TelemetryExporter {
    pub fn new(usage_stats: Arc<Mutex<UsageStats>>) -> Self {
        Self { usage_stats }
    }

    pub async fn export(&self) {
        let stats = self.usage_stats.lock().await;
        let data = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "usage": {
                "input_tokens": stats.input_tokens,
                "output_tokens": stats.output_tokens,
                "total_cost": stats.total_cost,
            },
            "status": "Session Completed"
        });

        let _ = fs::write(
            "telemetry.json",
            serde_json::to_string_pretty(&data).unwrap_or_default(),
        );
        println!("Telemetry exported to telemetry.json");
    }
}
