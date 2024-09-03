use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::errors::Result;
use super::room::Room;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }

    pub async fn make_room(&self) -> Result<String> {
        let id: String = Uuid::new_v4().to_string();
        let room = Room::new();
        self.rooms.lock().await.insert(id.clone(), room);
        Ok(id)
    }

    pub async fn delete_room(&self, id: String) -> Result<()> {
        self.rooms.lock().await.remove(&id);
        Ok(())
    }
}
