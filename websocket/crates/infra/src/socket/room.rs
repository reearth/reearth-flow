use std::collections::HashSet;
use std::sync::Mutex;
use tokio::sync::broadcast;

use super::errors::{Result, WsError};

pub struct Room {
    users: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

impl Room {
    pub fn new() -> Self {
        Room {
            users: Mutex::new(HashSet::new()),
            tx: broadcast::Sender::new(100),
        }
    }

    pub fn join(&mut self, user_id: String) -> Result<()> {
        self.users
            .try_lock()
            .or_else(|_| Err(WsError::WsError))?
            .insert(user_id);
        Ok(())
    }

    pub fn leave(&mut self, user_id: String) -> Result<()> {
        self.users
            .try_lock()
            .or_else(|_| Err(WsError::WsError))?
            .remove(&user_id);
        Ok(())
    }

    pub fn broadcast(&mut self, msg: String) -> Result<()> {
        self.tx.send(msg).or_else(|_| Err(WsError::WsError))?;
        Ok(())
    }
}
