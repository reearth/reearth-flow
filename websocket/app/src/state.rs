use super::errors::Result;
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
        let mut rooms = self.rooms.try_lock()?;
        rooms.insert(room_id, Room::new());
        Ok(())
    }

    pub fn delete_room(&self, id: String) -> Result<()> {
        let mut rooms = self.rooms.try_lock()?;
        rooms.remove(&id);
        Ok(())
    }
}
