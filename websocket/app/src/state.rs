use super::errors::Result;
use super::room::Room;
use redis::Client as RedisClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive()]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub redis_client: RedisClient,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            redis_client: RedisClient::open("redis://localhost:6379/0").unwrap(),
        }
    }
}

impl AppState {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let redis_client = RedisClient::open(redis_url)?;

        Ok(AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            redis_client,
        })
    }

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
