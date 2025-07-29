use crate::domain::entity::BroadcastGroup;
use crate::domain::repository::BroadcastRepository;
use crate::domain::value_object::{DocumentName, InstanceId};
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Infrastructure implementation of BroadcastRepository
pub struct BroadcastRepositoryImpl {
    groups: Arc<RwLock<HashMap<String, Arc<BroadcastGroup>>>>,
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Bytes>>>>,
    buffer_capacity: usize,
}

impl BroadcastRepositoryImpl {
    pub fn new(buffer_capacity: usize) -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            buffer_capacity: buffer_capacity.max(512),
        }
    }
}

#[async_trait]
impl BroadcastRepository for BroadcastRepositoryImpl {
    async fn create_group(
        &self,
        document_name: DocumentName,
        instance_id: InstanceId,
    ) -> Result<Arc<BroadcastGroup>> {
        let key = document_name.as_str().to_string();
        
        // Create the domain entity
        let group = Arc::new(BroadcastGroup::new(document_name, instance_id));
        
        // Create broadcast channel for this group
        let (sender, _) = broadcast::channel(self.buffer_capacity);
        
        // Store both group and channel
        {
            let mut groups = self.groups.write().await;
            let mut channels = self.channels.write().await;
            
            groups.insert(key.clone(), group.clone());
            channels.insert(key, sender);
        }
        
        Ok(group)
    }

    async fn get_group(
        &self,
        document_name: &DocumentName,
    ) -> Result<Option<Arc<BroadcastGroup>>> {
        let key = document_name.as_str();
        let groups = self.groups.read().await;
        Ok(groups.get(key).cloned())
    }

    async fn remove_group(&self, document_name: &DocumentName) -> Result<()> {
        let key = document_name.as_str();
        
        let mut groups = self.groups.write().await;
        let mut channels = self.channels.write().await;
        
        groups.remove(key);
        channels.remove(key);
        
        Ok(())
    }

    async fn broadcast_message(
        &self,
        document_name: &DocumentName,
        message: Bytes,
    ) -> Result<()> {
        let key = document_name.as_str();
        let channels = self.channels.read().await;
        
        if let Some(sender) = channels.get(key) {
            // Ignore error if no receivers (channel is empty)
            let _ = sender.send(message);
        }
        
        Ok(())
    }

    async fn subscribe(
        &self,
        document_name: &DocumentName,
    ) -> Result<broadcast::Receiver<Bytes>> {
        let key = document_name.as_str();
        let channels = self.channels.read().await;
        
        if let Some(sender) = channels.get(key) {
            Ok(sender.subscribe())
        } else {
            Err(anyhow::anyhow!("Broadcast group not found: {}", key))
        }
    }
}
