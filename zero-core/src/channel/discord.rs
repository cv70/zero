use std::sync::Arc;

use async_trait::async_trait;

use crate::channel::{Channel, ChannelKind, ChannelMessage, WebhookPayload, MessageQueue, Persistence};

pub struct DiscordChannel {
    pub id: String,
    pub queue: Arc<MessageQueue>,
    pub persistence: Arc<Persistence>,
}

impl DiscordChannel {
    pub fn new(id: String, queue: Arc<MessageQueue>, persistence: Arc<Persistence>) -> Self {
        Self { id, queue, persistence }
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Simple background processor similar to Telegram
        let q = self.queue.clone();
        let p = self.persistence.clone();
        tokio::spawn(async move {
            loop {
                if let Some(msg) = q.dequeue().await {
                    let _ = p.save_message(&msg).await;
                } else {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        });
        Ok(())
    }

    fn kind(&self) -> ChannelKind {
        ChannelKind::Discord
    }

    async fn send(&self, msg: ChannelMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.queue.enqueue(msg).await;
        Ok(())
    }

    async fn handle_webhook(&self, payload: WebhookPayload) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = ChannelMessage {
            id: uuid::Uuid::new_v4().to_string(),
            channel: ChannelKind::Discord,
            content: payload.content,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: payload.headers,
        };
        self.queue.enqueue(msg).await;
        Ok(())
    }
}

use std::time::{SystemTime, UNIX_EPOCH};
