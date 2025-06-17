use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use yrs::Doc;

#[derive(Clone, Debug)]
pub struct MockRedisStore {
    streams: Arc<Mutex<HashMap<String, Vec<Bytes>>>>,
    locks: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for MockRedisStore {
    fn default() -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new())),
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockRedisStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_stream_data(&self, doc_id: &str, data: Bytes) {
        let mut streams = self.streams.lock().unwrap();
        streams.entry(doc_id.to_string()).or_default().push(data);
    }

    pub async fn acquire_doc_lock(&self, lock_id: &str, instance_id: &str) -> anyhow::Result<bool> {
        let mut locks = self.locks.lock().unwrap();
        if locks.contains_key(lock_id) {
            Ok(false)
        } else {
            locks.insert(lock_id.to_string(), instance_id.to_string());
            Ok(true)
        }
    }

    pub async fn release_doc_lock(&self, lock_id: &str, instance_id: &str) -> anyhow::Result<()> {
        let mut locks = self.locks.lock().unwrap();
        if let Some(current_instance) = locks.get(lock_id) {
            if current_instance == instance_id {
                locks.remove(lock_id);
            }
        }
        Ok(())
    }

    pub async fn read_all_stream_data(&self, doc_id: &str) -> anyhow::Result<Vec<Bytes>> {
        let streams = self.streams.lock().unwrap();
        Ok(streams.get(doc_id).cloned().unwrap_or_default())
    }
}

#[derive(Clone)]
pub struct MockGcsStore {
    docs: Arc<RwLock<HashMap<String, Doc>>>,
}

impl Default for MockGcsStore {
    fn default() -> Self {
        Self {
            docs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl MockGcsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn load_doc_v2(&self, doc_name: &str) -> anyhow::Result<Doc> {
        let docs = self.docs.read().await;
        if let Some(doc) = docs.get(doc_name) {
            Ok(doc.clone())
        } else {
            Ok(Doc::new())
        }
    }

    pub async fn flush_doc_v2(&self, doc_name: &str, doc: &Doc) -> anyhow::Result<()> {
        let mut docs = self.docs.write().await;
        docs.insert(doc_name.to_string(), doc.clone());
        Ok(())
    }
}

impl std::fmt::Debug for MockGcsStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockGcsStore").finish()
    }
}
