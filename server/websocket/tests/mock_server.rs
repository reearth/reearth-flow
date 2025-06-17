use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use yrs::Doc;

#[derive(Clone, Debug)]
pub struct MockRedisStore {
    streams: Arc<Mutex<HashMap<String, Vec<Bytes>>>>,
    locks: Arc<Mutex<HashMap<String, String>>>,
    lock_should_fail: Arc<Mutex<bool>>,
}

impl Default for MockRedisStore {
    fn default() -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new())),
            locks: Arc::new(Mutex::new(HashMap::new())),
            lock_should_fail: Arc::new(Mutex::new(false)),
        }
    }
}

impl MockRedisStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_lock_should_fail(&self, should_fail: bool) {
        let mut lock_should_fail = self.lock_should_fail.lock().unwrap();
        *lock_should_fail = should_fail;
    }

    pub fn add_stream_data(&self, doc_id: &str, data: Bytes) {
        let mut streams = self.streams.lock().unwrap();
        streams.entry(doc_id.to_string()).or_default().push(data);
    }

    pub fn clear_stream(&self, doc_id: &str) {
        let mut streams = self.streams.lock().unwrap();
        streams.remove(doc_id);
    }

    pub async fn acquire_doc_lock(&self, lock_id: &str, instance_id: &str) -> anyhow::Result<bool> {
        let should_fail = *self.lock_should_fail.lock().unwrap();
        if should_fail {
            return Ok(false);
        }

        let mut locks = self.locks.lock().unwrap();
        if locks.contains_key(lock_id) {
            Ok(false) // Lock already acquired
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
    should_fail_operations: Arc<Mutex<bool>>,
}

impl Default for MockGcsStore {
    fn default() -> Self {
        Self {
            docs: Arc::new(RwLock::new(HashMap::new())),
            should_fail_operations: Arc::new(Mutex::new(false)),
        }
    }
}

impl MockGcsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_should_fail(&self, should_fail: bool) {
        let mut should_fail_ops = self.should_fail_operations.lock().unwrap();
        *should_fail_ops = should_fail;
    }

    pub async fn load_doc_v2(
        &self,
        doc_name: &str,
    ) -> Result<Doc, Box<dyn std::error::Error + Send + Sync>> {
        let should_fail = *self.should_fail_operations.lock().unwrap();
        if should_fail {
            return Err("Mock GCS operation failed".into());
        }

        let docs = self.docs.read().await;
        if let Some(doc) = docs.get(doc_name) {
            Ok(doc.clone())
        } else {
            Ok(Doc::new())
        }
    }

    pub async fn flush_doc_v2(
        &self,
        doc_name: &str,
        doc: &Doc,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let should_fail = *self.should_fail_operations.lock().unwrap();
        if should_fail {
            return Err("Mock GCS flush operation failed".into());
        }

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
