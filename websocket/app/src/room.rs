use std::collections::HashSet;
use std::sync::Arc;
use yrs::Doc;

use tokio::sync::{broadcast, Mutex};

pub struct Room {
    users: Arc<Mutex<HashSet<String>>>,
    _tx: Arc<broadcast::Sender<String>>,
    doc: Arc<Doc>,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            users: Arc::new(Mutex::new(HashSet::new())),
            _tx: Arc::new(broadcast::Sender::new(100)),
            doc: Arc::new(Doc::new()),
        }
    }
}

impl Room {
    pub fn new() -> Self {
        Room::default()
    }

    pub async fn join(&self, user_id: String) {
        self.users.lock().await.insert(user_id);
    }

    pub async fn _leave(&self, user_id: String) {
        self.users.lock().await.remove(&user_id);
    }

    pub fn _broadcast(&self, msg: String) -> Result<(), broadcast::error::SendError<String>> {
        self._tx.send(msg)?;
        Ok(())
    }

    pub fn get_doc(&self) -> Arc<Doc> {
        self.doc.clone()
    }
}
