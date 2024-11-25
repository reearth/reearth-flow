use flow_websocket_services::manage_project_edit_session::SessionCommand;
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
    project_id: Option<String>,
}

impl WebSocketQuery {
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn project_id(&self) -> Option<String> {
        self.project_id.clone()
    }

    pub fn _update_project_id(&mut self, project_id: Option<String>) {
        self.project_id = project_id;
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MessageType(u8);

impl MessageType {
    pub const UPDATE: MessageType = MessageType(1);
    pub const SYNC: MessageType = MessageType(2);

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::UPDATE),
            2 => Some(Self::SYNC),
            _ => None,
        }
    }

    pub fn _as_byte(&self) -> u8 {
        self.0
    }

    pub fn _is_update(&self) -> bool {
        *self == Self::UPDATE
    }

    pub fn _is_sync(&self) -> bool {
        *self == Self::SYNC
    }
}

pub fn parse_message(data: &[u8]) -> Option<(MessageType, &[u8])> {
    if data.is_empty() {
        return None;
    }

    MessageType::from_byte(data[0]).map(|msg_type| (msg_type, &data[1..]))
}
