use crate::integration::envelope::Envelope;
use serde_json::Value;

#[async_trait::async_trait]
pub trait Adapter: Send + Sync {
    fn name(&self) -> &'static str;
    async fn process(&self, msg: Envelope<Value>) -> Result<Envelope<Value>, Box<dyn std::error::Error + Send + Sync>>;
}

// Simple helper to boxed adapter
pub type DynAdapter = Box<dyn Adapter>;
