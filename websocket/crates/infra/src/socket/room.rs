use std::collections::HashSet;
use std::sync::Arc;
use yrs::Doc;

use super::errors::{Result, WsError};
use tokio::sync::{broadcast, Mutex};

pub struct Room {
    users: Arc<Mutex<HashSet<String>>>,
    tx: Arc<broadcast::Sender<String>>,
    doc: Arc<Doc>,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            users: Arc::new(Mutex::new(HashSet::new())),
            tx: Arc::new(broadcast::Sender::new(100)),
            doc: Arc::new(Doc::new()),
        }
    }
}

impl Room {
    pub fn new() -> Self {
        Room::default()
    }

    pub async fn join(&self, user_id: String) -> Result<()> {
        self.users.lock().await.insert(user_id);
        Ok(())
    }

    pub async fn leave(&self, user_id: String) -> Result<()> {
        self.users.lock().await.remove(&user_id);
        Ok(())
    }

    pub fn broadcast(&self, msg: String) -> Result<()> {
        self.tx.send(msg).map_err(|_| WsError::WsError)?;
        Ok(())
    }

    pub fn get_doc(&self) -> Arc<Doc> {
        self.doc.clone()
    }
}
