use crate::shared::errors::AppError;
use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(value: impl Into<String>) -> AppResult<Self> {
        let value = value.into();
        ensure_not_empty(&value, "session_id")?;
        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientId(String);

impl ClientId {
    pub fn new(value: impl Into<String>) -> AppResult<Self> {
        let value = value.into();
        ensure_not_empty(&value, "client_id")?;
        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub doc_id: String,
    pub client_id: ClientId,
    pub session_id: SessionId,
    pub user_token: Option<String>,
}

impl ConnectionInfo {
    pub fn new(
        doc_id: impl Into<String>,
        client_id: ClientId,
        session_id: SessionId,
        user_token: Option<String>,
    ) -> AppResult<Self> {
        let doc_id = doc_id.into();
        ensure_not_empty(&doc_id, "doc_id")?;
        Ok(Self {
            doc_id,
            client_id,
            session_id,
            user_token,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub info: ConnectionInfo,
    pub active: bool,
}

impl ConnectionState {
    pub fn new(info: ConnectionInfo) -> Self {
        Self { info, active: true }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionSummary {
    pub doc_id: String,
    pub active_connections: usize,
}

impl ConnectionSummary {
    pub fn new(doc_id: impl Into<String>, active_connections: usize) -> Self {
        Self {
            doc_id: doc_id.into(),
            active_connections,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwarenessUpdate {
    pub doc_id: String,
    pub payload: Vec<u8>,
}

impl AwarenessUpdate {
    pub fn new(doc_id: impl Into<String>, payload: Vec<u8>) -> AppResult<Self> {
        let doc_id = doc_id.into();
        ensure_not_empty(&doc_id, "doc_id")?;
        if payload.is_empty() {
            return Err(AppError::invalid_input("awareness payload cannot be empty"));
        }

        Ok(Self { doc_id, payload })
    }
}
