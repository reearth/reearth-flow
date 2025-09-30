use chrono::{DateTime, Utc};

use crate::domain::entities::ws::{ClientId, ConnectionInfo, SessionId};
use crate::shared::result::AppResult;

#[derive(Debug, Clone)]
pub struct ConnectionEstablished {
    pub occurred_at: DateTime<Utc>,
    pub session_id: SessionId,
    pub client_id: ClientId,
    pub doc_id: String,
}

impl ConnectionEstablished {
    pub fn new(info: &ConnectionInfo) -> AppResult<Self> {
        Ok(Self {
            occurred_at: Utc::now(),
            session_id: info.session_id.clone(),
            client_id: info.client_id.clone(),
            doc_id: info.doc_id.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionClosed {
    pub occurred_at: DateTime<Utc>,
    pub session_id: SessionId,
    pub client_id: ClientId,
    pub doc_id: String,
}

impl ConnectionClosed {
    pub fn new(info: &ConnectionInfo) -> AppResult<Self> {
        Ok(Self {
            occurred_at: Utc::now(),
            session_id: info.session_id.clone(),
            client_id: info.client_id.clone(),
            doc_id: info.doc_id.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AwarenessSynced {
    pub occurred_at: DateTime<Utc>,
    pub doc_id: String,
    pub client_id: ClientId,
}

impl AwarenessSynced {
    pub fn new(doc_id: impl Into<String>, client_id: ClientId) -> Self {
        Self {
            occurred_at: Utc::now(),
            doc_id: doc_id.into(),
            client_id,
        }
    }
}
