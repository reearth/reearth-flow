use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// 消息类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Sync,
    Awareness,
    Broadcast,
}

/// 广播消息实体
#[derive(Debug, Clone)]
pub struct BroadcastMessage {
    pub message_type: MessageType,
    pub data: Bytes,
    pub origin: Option<String>,
}

impl BroadcastMessage {
    pub fn new(message_type: MessageType, data: Bytes) -> Self {
        Self {
            message_type,
            data,
            origin: None,
        }
    }

    pub fn with_origin(mut self, origin: String) -> Self {
        self.origin = Some(origin);
        self
    }
}
