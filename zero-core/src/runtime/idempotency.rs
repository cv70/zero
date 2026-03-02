use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct IdempotencyStore {
    inner: Arc<RwLock<HashMap<String, StepCacheEntry>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepCacheEntry {
    pub step_id: String,
    pub output: String,
}

impl IdempotencyStore {
    pub async fn get(&self, key: &str) -> Option<StepCacheEntry> {
        self.inner.read().await.get(key).cloned()
    }

    pub async fn put(&self, key: String, value: StepCacheEntry) {
        self.inner.write().await.insert(key, value);
    }
}
