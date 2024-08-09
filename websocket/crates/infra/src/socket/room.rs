use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use super::errors::{Result, WsError};

pub struct Room {
    users: Arc<Mutex<HashSet<String>>>,
    tx: Arc<broadcast::Sender<String>>,
}

impl Room {
    pub fn new() -> Self {
        Room {
            users: Arc::new(Mutex::new(HashSet::new())),
            tx: Arc::new(broadcast::Sender::new(100)),
        }
    }

    pub fn join(&mut self, user_id: String) -> Result<()> {
        self.users
            .try_lock()
            .map_err(|_| WsError::WsError)?
            .insert(user_id);
        Ok(())
    }

    pub fn leave(&mut self, user_id: String) -> Result<()> {
        self.users
            .try_lock()
            .map_err(|_| WsError::WsError)?
            .remove(&user_id);
        Ok(())
    }

    pub fn broadcast(&mut self, msg: String) -> Result<()> {
        self.tx.send(msg).or_else(|_| Err(WsError::WsError))?;
        Ok(())
    }
}
