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
}
