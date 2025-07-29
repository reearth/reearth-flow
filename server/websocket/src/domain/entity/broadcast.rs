use crate::domain::entity::connection_id::ConnectionId;
use crate::domain::entity::document_name::DocumentName;
use crate::domain::entity::instance_id::InstanceId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Domain entity representing a broadcast group for collaborative document editing
#[derive(Debug)]
pub struct BroadcastGroup {
    document_name: DocumentName,
    instance_id: InstanceId,
    connections: AtomicUsize,
    active_connections: HashMap<ConnectionId, Connection>,
}

#[derive(Debug, Clone)]
pub struct Connection {
    id: ConnectionId,
    user_token: Option<String>,
    connected_at: std::time::SystemTime,
}

impl BroadcastGroup {
    pub fn new(document_name: DocumentName, instance_id: InstanceId) -> Self {
        Self {
            document_name,
            instance_id,
            connections: AtomicUsize::new(0),
            active_connections: HashMap::new(),
        }
    }

    pub fn document_name(&self) -> &DocumentName {
        &self.document_name
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::Relaxed)
    }

    pub fn increment_connections(&self) -> usize {
        self.connections.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn decrement_connections(&self) -> usize {
        let prev = self.connections.fetch_sub(1, Ordering::Relaxed);
        if prev > 0 {
            prev - 1
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.connection_count() == 0
    }
}

impl Connection {
    pub fn new(id: ConnectionId, user_token: Option<String>) -> Self {
        Self {
            id,
            user_token,
            connected_at: std::time::SystemTime::now(),
        }
    }

    pub fn id(&self) -> &ConnectionId {
        &self.id
    }

    pub fn user_token(&self) -> Option<&str> {
        self.user_token.as_deref()
    }

    pub fn connected_at(&self) -> std::time::SystemTime {
        self.connected_at
    }
}
