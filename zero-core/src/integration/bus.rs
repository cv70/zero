use serde::Serialize;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize)]
pub struct BusMessage {
    pub topic: String,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub created_at: i64,
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<BusMessage>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let (tx, _rx) = broadcast::channel(buffer);
        EventBus { sender: tx }
    }

    pub fn subscriber(&self) -> BusReceiver {
        self.sender.subscribe()
    }

    pub async fn publish(&self, msg: BusMessage) {
        let _ = self.sender.send(msg);
    }
}

pub type BusReceiver = broadcast::Receiver<BusMessage>;