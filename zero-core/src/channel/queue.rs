use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::channel::{ChannelMessage};

pub struct MessageQueue {
    inner: Arc<Mutex<mpsc::Receiver<ChannelMessage>>>,
    sender: mpsc::Sender<ChannelMessage>,
}

impl MessageQueue {
    // Create a new queue with given buffer size. Returns (queue_handle, sender) so callers can enqueue.
    pub fn new(buffer: usize) -> (Self, mpsc::Sender<ChannelMessage>) {
        let (tx, rx) = mpsc::channel(buffer);
        let queue = MessageQueue {
            inner: Arc::new(Mutex::new(rx)),
            sender: tx.clone(),
        };
        (queue, tx)
    }

    pub async fn enqueue(&self, msg: ChannelMessage) {
        let _ = self.sender.send(msg).await;
    }

    pub async fn dequeue(&self) -> Option<ChannelMessage> {
        let mut rx = self.inner.lock().await;
        rx.recv().await
    }
}
