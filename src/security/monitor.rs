use log::info;
use std::time::Duration;
use tokio::time;

// Lightweight, example security monitor that emits heartbeat logs.
pub async fn start_monitoring() {
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        info!("security-monitor: heartbeat OK");
    }
}
