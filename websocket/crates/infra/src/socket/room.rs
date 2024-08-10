use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use yrs::Doc;

use super::errors::{Result, WsError};

pub struct Room {
    users: Arc<Mutex<HashSet<String>>>,
    tx: Arc<broadcast::Sender<String>>,
    doc: Arc<Doc>,
}

impl Room {
    pub fn new() -> Self {
        Room {
            users: Arc::new(Mutex::new(HashSet::new())),
            tx: Arc::new(broadcast::Sender::new(100)),
            doc: Arc::new(Doc::new()),
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

    pub fn get_doc(&self) -> Arc<Doc> {
        self.doc.clone()
    }
}
