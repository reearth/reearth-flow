use super::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yrs::{
    updates::decoder::Decode, updates::encoder::Encode, Doc, ReadTxn,
    StateVector as YrsStateVector, Transact, Update,
};

#[derive(Debug, Clone)]
pub struct Document {
    id: DocumentId,
    doc: Doc,
    latest_version: UpdateVersion,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Document {
    pub fn new(id: DocumentId) -> Self {
        let now = Utc::now();
        Self {
            id,
            doc: Doc::new(),
            latest_version: UpdateVersion::new(0),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_doc(id: DocumentId, doc: Doc, version: UpdateVersion) -> Self {
        let now = Utc::now();
        Self {
            id,
            doc,
            latest_version: version,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    pub fn doc(&self) -> &Doc {
        &self.doc
    }

    pub fn latest_version(&self) -> &UpdateVersion {
        &self.latest_version
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn apply_update(&mut self, update: &DocumentUpdate) -> Result<(), String> {
        let mut txn = self.doc.transact_mut();
        let yrs_update = Update::decode_v1(&update.data)
            .map_err(|e| format!("Failed to decode update: {}", e))?;
        match txn.apply_update(yrs_update) {
            Ok(_) => {
                self.latest_version = update.version.clone();
                self.updated_at = Utc::now();
                Ok(())
            }
            Err(e) => Err(format!("Failed to apply update: {}", e)),
        }
    }

    pub fn get_state_vector(&self) -> StateVector {
        let txn = self.doc.transact();
        let sv = txn.state_vector().encode_v1();
        StateVector::new(sv)
    }

    pub fn get_diff_update(&self, state_vector: &StateVector) -> Vec<u8> {
        let txn = self.doc.transact();
        let sv = YrsStateVector::decode_v1(&state_vector.0).unwrap_or_default();
        txn.encode_diff_v1(&sv)
    }

    pub fn get_state_as_update(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&YrsStateVector::default())
    }
}

#[derive(Debug, Clone)]
pub struct DocumentSession {
    id: SessionId,
    document_id: DocumentId,
    user_id: Option<UserId>,
    connected_at: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    is_active: bool,
}

impl DocumentSession {
    pub fn new(document_id: DocumentId, user_id: Option<UserId>) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::generate(),
            document_id,
            user_id,
            connected_at: now,
            last_seen: now,
            is_active: true,
        }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    pub fn user_id(&self) -> Option<&UserId> {
        self.user_id.as_ref()
    }

    pub fn connected_at(&self) -> DateTime<Utc> {
        self.connected_at
    }

    pub fn last_seen(&self) -> DateTime<Utc> {
        self.last_seen
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    pub fn disconnect(&mut self) {
        self.is_active = false;
        self.last_seen = Utc::now();
    }

    pub fn is_expired(&self, timeout_seconds: i64) -> bool {
        let now = Utc::now();
        (now - self.last_seen).num_seconds() > timeout_seconds
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    name: Option<String>,
    metadata: HashMap<String, String>,
}

impl User {
    pub fn new(id: UserId) -> Self {
        Self {
            id,
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_name(id: UserId, name: String) -> Self {
        Self {
            id,
            name: Some(name),
            metadata: HashMap::new(),
        }
    }

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|v| v.as_str())
    }
}
