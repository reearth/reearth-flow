use flow_websocket_services::SessionCommand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum Event {
    Create { room_id: String },
    Join { room_id: String },
    Leave,
    Emit { data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMessage {
    pub event: Event,
    pub session_command: Option<SessionCommand>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketQuery {
    token: String,
    user_id: String,
}

impl WebSocketQuery {
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Update = 1,
    Sync = 2,
}

impl MessageType {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::Update),
            2 => Some(Self::Sync),
            _ => None,
        }
    }

    pub fn _as_byte(&self) -> u8 {
        *self as u8
    }
}

/// Parses a binary message according to the Flow protocol format:
/// - Byte 0: Message type (1 = UPDATE, 2 = SYNC)
/// - Bytes 1+: Message payload
///
/// Returns None if the input is empty or has an invalid message type.
/// Returns Some((message_type, payload)) on successful parsing.
pub fn parse_message(data: &[u8]) -> Option<(MessageType, &[u8])> {
    // Ensure we have at least one byte for the message type
    let type_byte = *data.first()?;

    // Parse and validate message type
    let msg_type = MessageType::from_byte(type_byte)?;

    // Return message type and remaining payload
    Some((msg_type, &data[1..]))
}
