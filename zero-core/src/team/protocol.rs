/// Team communication protocol
use serde::{Deserialize, Serialize};

/// Message type for inter-agent communication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "task_request")]
    TaskRequest,
    #[serde(rename = "task_result")]
    TaskResult,
    #[serde(rename = "status_update")]
    StatusUpdate,
    #[serde(rename = "heartbeat")]
    Heartbeat,
}

/// Team message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    pub message_type: MessageType,
    pub from_agent: String,
    pub to_agent: String,
    pub payload: String,
    pub timestamp: String,
}

impl TeamMessage {
    pub fn new(
        message_type: MessageType,
        from_agent: String,
        to_agent: String,
        payload: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        Self {
            message_type,
            from_agent,
            to_agent,
            payload,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_message() {
        let msg = TeamMessage::new(
            MessageType::TaskRequest,
            "agent_1".to_string(),
            "agent_2".to_string(),
            "task_data".to_string(),
        );

        assert_eq!(msg.message_type, MessageType::TaskRequest);
        assert_eq!(msg.from_agent, "agent_1");
    }

    #[test]
    fn test_message_serialization() {
        let msg = TeamMessage::new(
            MessageType::Heartbeat,
            "a1".to_string(),
            "a2".to_string(),
            "".to_string(),
        );

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: TeamMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.from_agent, "a1");
    }
}
