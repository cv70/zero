use zero_core::integration::bus::{EventBus, BusMessage}; // Use explicit path if crate name is zero_core
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

#[tokio::test]
async fn test_bus_publish_subscribe() {
    let bus = EventBus::new(16);
    let mut rx = bus.publisher().subscribe();
    let msg = BusMessage {
        topic: "test.topic".to_string(),
        payload: serde_json::json!({"hello": "world"}),
        correlation_id: Some("corr-1".to_string()),
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
    };
    bus.publish(msg.clone()).await;
    let received = rx.recv().await.unwrap();
    assert_eq!(received.topic, msg.topic);
}
