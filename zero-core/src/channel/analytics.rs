use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::channel::ChannelKind;

static TOTAL_MESSAGES: AtomicU64 = AtomicU64::new(0);

pub struct Analytics;

impl Analytics {
    pub fn new() -> Self {
        Self
    }
    pub fn record(_kind: ChannelKind) {
        TOTAL_MESSAGES.fetch_add(1, Ordering::SeqCst);
    }
    pub fn total() -> u64 {
        TOTAL_MESSAGES.load(Ordering::SeqCst)
    }
}
