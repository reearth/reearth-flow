use super::errors::{Result, WsError};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub struct Room {
    users: Arc<Mutex<HashSet<String>>>,
    tx: Arc<broadcast::Sender<String>>,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            users: Arc::new(Mutex::new(HashSet::new())),
            tx: Arc::new(broadcast::channel(100).0), // initialize broadcast channel with a capacity of 100
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
}
