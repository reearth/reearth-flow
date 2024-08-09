use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::errors::{Result, WsError};
use super::room::Room;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn make_room(&mut self) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let room = Room::new();
        self.rooms
            .try_lock()
            .map_err(|_| WsError::WsError)?
            .insert(id.clone(), room);
        Ok(id)
    }

    pub fn delete_room(&mut self, id: String) -> Result<()> {
        self.rooms
            .try_lock()
            .map_err(|_| WsError::WsError)?
            .remove(&id);
        Ok(())
    }
}
