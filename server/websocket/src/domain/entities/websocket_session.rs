use std::collections::HashMap;

use crate::domain::entities::ws::{ConnectionInfo, ConnectionState, ConnectionSummary};
use crate::shared::errors::AppError;
use crate::shared::result::AppResult;

#[derive(Debug, Default)]
pub struct WebsocketSession {
    doc_id: String,
    connections: HashMap<String, ConnectionState>,
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
        })
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
        Ok(())
    }

    pub fn disconnect(&mut self, session_id: &str) -> AppResult<()> {
        let state = self
            .connections
            .get_mut(session_id)
            .ok_or_else(|| AppError::not_found("connection not found"))?;
        state.deactivate();
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
}
