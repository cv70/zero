use std::collections::HashMap;

use crate::channel::{Channel, ChannelKind, WebhookPayload};

// Simple helper to route a webhook payload to the appropriate channel if available.
pub struct WebhookHandler;

impl WebhookHandler {
    pub async fn route(payload: WebhookPayload, channel: &dyn Channel) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        channel.handle_webhook(payload).await
    }
}
