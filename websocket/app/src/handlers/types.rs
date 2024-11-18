use flow_websocket_services::manage_project_edit_session::SessionCommand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum Event {
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
    pub token: String,
    pub user_id: String,
    pub project_id: Option<String>,
}
