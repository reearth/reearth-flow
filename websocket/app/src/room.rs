use std::collections::HashSet;
use std::sync::Arc;
use yrs::Doc;

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

    /// Adds a user to the room
    pub async fn join(&self, user_id: String) {
        self.users.lock().await.insert(user_id);
    }

    /// Removes a user from the room
    pub async fn leave(&self, user_id: String) {
        self.users.lock().await.remove(&user_id);
    }

    /// Broadcasts a message to all users in the room
    pub fn broadcast(&self, msg: String) -> Result<(), broadcast::error::SendError<String>> {
        self.tx.send(msg)?;
        Ok(())
    }

    /// Returns a clone of the room's document
    pub fn get_doc(&self) -> Arc<Doc> {
        self.doc.clone()
    }

    /// Returns a list of user IDs currently in the room
    pub async fn get_users(&self) -> HashSet<String> {
        self.users.lock().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_join_and_get_users() {
        let room = Room::new();
        let user_id = "user1".to_string();

        room.join(user_id.clone()).await;
        let users = room.get_users().await;

        assert!(users.contains(&user_id));
        assert_eq!(users.len(), 1);
    }

    #[tokio::test]
    async fn test_leave() {
        let room = Room::new();
        let user_id = "user1".to_string();

        room.join(user_id.clone()).await;
        room.leave(user_id.clone()).await;

        let users = room.get_users().await;
        assert!(!users.contains(&user_id));
        assert_eq!(users.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_users() {
        let room = Room::new();
        let user1 = "user1".to_string();
        let user2 = "user2".to_string();

        room.join(user1.clone()).await;
        room.join(user2.clone()).await;

        let users = room.get_users().await;
        assert_eq!(users.len(), 2);
        assert!(users.contains(&user1));
        assert!(users.contains(&user2));
    }

    #[tokio::test]
    async fn test_broadcast() {
        let room = Room::new();
        let msg = "test message".to_string();

        let _rx = room.tx.subscribe();

        let result = room.broadcast(msg);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_doc() {
        let room = Room::new();
        let doc = room.get_doc();

        assert!(Arc::strong_count(&doc) > 1);
    }
}
