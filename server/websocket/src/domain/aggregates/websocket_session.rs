use std::collections::HashMap;

use crate::domain::entities::ws::{ClientId, ConnectionInfo, ConnectionState, ConnectionSummary};
use crate::domain::events::{AwarenessSynced, ConnectionClosed, ConnectionEstablished};
use crate::shared::errors::AppError;
use crate::shared::result::AppResult;

#[derive(Debug, Default)]
pub struct WebsocketSession {
    doc_id: String,
    connections: HashMap<String, ConnectionState>,
    events: Vec<WebsocketEvent>,
}

#[derive(Debug, Clone)]
pub enum WebsocketEvent {
    Established(ConnectionEstablished),
    Closed(ConnectionClosed),
    Awareness(AwarenessSynced),
}

impl WebsocketSession {
    pub fn new(doc_id: impl Into<String>) -> AppResult<Self> {
        let doc_id = doc_id.into();
        if doc_id.trim().is_empty() {
            return Err(AppError::invalid_input("doc_id cannot be empty"));
        }
        Ok(Self {
            doc_id,
            connections: HashMap::new(),
            events: Vec::new(),
        })
    }

    pub fn apply(&mut self, event: WebsocketEvent) {
        match &event {
            WebsocketEvent::Established(established) => {
                let key = established.session_id.value().to_string();
                if let Some(state) = self.connections.get_mut(&key) {
                    state.active = true;
                }
            }
            WebsocketEvent::Closed(closed) => {
                let key = closed.session_id.value().to_string();
                if let Some(state) = self.connections.get_mut(&key) {
                    state.deactivate();
                }
            }
            WebsocketEvent::Awareness(_) => {}
        }
        self.events.push(event);
    }

    pub fn connect(&mut self, info: ConnectionInfo) -> AppResult<()> {
        if info.doc_id != self.doc_id {
            return Err(AppError::invalid_input("connection doc_id mismatch"));
        }
        let key = info.session_id.value().to_string();
        let state = self
            .connections
            .entry(key)
            .or_insert_with(|| ConnectionState::new(info.clone()));
        state.active = true;
        let event = ConnectionEstablished::new(&info)?;
        self.events.push(WebsocketEvent::Established(event));
        Ok(())
    }

    pub fn disconnect(&mut self, session_id: &str) -> AppResult<()> {
        let state = self
            .connections
            .get_mut(session_id)
            .ok_or_else(|| AppError::not_found("connection not found"))?;
        state.deactivate();
        let info = state.info.clone();
        let event = ConnectionClosed::new(&info)?;
        self.events.push(WebsocketEvent::Closed(event));
        Ok(())
    }

    pub fn awareness_synced(&mut self, doc_id: &str, client_id: &str) -> AppResult<()> {
        if doc_id != self.doc_id {
            return Err(AppError::invalid_input("awareness doc_id mismatch"));
        }
        let client = ClientId::new(client_id.to_string())?;
        let event = AwarenessSynced::new(doc_id.to_string(), client);
        self.events.push(WebsocketEvent::Awareness(event));
        Ok(())
    }

    pub fn active_connections(&self) -> ConnectionSummary {
        let active = self
            .connections
            .values()
            .filter(|state| state.active)
            .count();
        ConnectionSummary::new(&self.doc_id, active)
    }

    pub fn events(&self) -> &[WebsocketEvent] {
        &self.events
    }

    pub fn drain_events(&mut self) -> Vec<WebsocketEvent> {
        std::mem::take(&mut self.events)
    }
}
