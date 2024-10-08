use super::errors::{Result, WsError};
use super::room::Room;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive()]
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
    pub fn make_room(&self, room_id: String) -> Result<()> {
        let room = Room::new();
        self.rooms
            .try_lock()
            .map_err(|_| WsError::Error)?
            .insert(room_id, room);
        Ok(())
    }

    pub fn _delete_room(&self, id: String) -> Result<()> {
        self.rooms
            .try_lock()
            .map_err(|_| WsError::Error)?
            .remove(&id);
        Ok(())
    }
}
