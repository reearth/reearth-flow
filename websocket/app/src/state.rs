use super::room::Room;
use crate::errors::WsError;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use flow_websocket_infra::persistence::project_repository::ProjectRedisRepository;
use flow_websocket_infra::persistence::redis::flow_project_redis_data_manager::FlowProjectRedisDataManager;
#[cfg(feature = "gcs-storage")]
#[allow(unused_imports)]
use flow_websocket_infra::persistence::ProjectGcsRepository;
#[cfg(feature = "local-storage")]
#[allow(unused_imports)]
use flow_websocket_infra::persistence::ProjectLocalRepository;
use flow_websocket_services::manage_project_edit_session::ManageEditSessionService;
use flow_websocket_services::manage_project_edit_session::SessionCommand;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::debug;
use tracing::error;

#[cfg(feature = "gcs-storage")]
#[cfg(not(feature = "local-storage"))]
pub type ProjectStorageRepository = ProjectGcsRepository;

#[cfg(feature = "local-storage")]
pub type ProjectStorageRepository = ProjectLocalRepository;

type SessionService = ManageEditSessionService<
    ProjectRedisRepository,
    ProjectStorageRepository,
    FlowProjectRedisDataManager,
>;

const DEFAULT_REDIS_URL: &str = "redis://localhost:6379/0";
const CHANNEL_BUFFER_SIZE: usize = 32;
#[cfg(feature = "local-storage")]
const DEFAULT_LOCAL_STORAGE_PATH: &str = "./local_storage";

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub redis_pool: Pool<RedisConnectionManager>,
    pub storage: Arc<ProjectStorageRepository>,
    pub session_repo: Arc<ProjectRedisRepository>,
    pub service: Arc<SessionService>,
    pub command_tx: mpsc::Sender<SessionCommand>,
}

impl AppState {
    pub async fn new(redis_url: Option<String>) -> Result<Self, WsError> {
        let redis_url = redis_url.unwrap_or_else(|| {
            std::env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string())
        });

        // Initialize Redis connection pool
        let manager = RedisConnectionManager::new(&*redis_url)?;
        let redis_pool = Pool::builder().build(manager).await?;

        // Initialize storage based on feature
        #[cfg(feature = "local-storage")]
        #[allow(unused_variables)]
        let storage =
            Arc::new(ProjectStorageRepository::new(DEFAULT_LOCAL_STORAGE_PATH.into()).await?);

        #[cfg(feature = "gcs-storage")]
        #[cfg(not(feature = "local-storage"))]
        let gcs_bucket =
            std::env::var("GCS_BUCKET_NAME").expect("GCS_BUCKET_NAME must be provided");

        #[cfg(feature = "gcs-storage")]
        #[cfg(not(feature = "local-storage"))]
        #[allow(unused_variables)]
        let storage = Arc::new(ProjectStorageRepository::new(gcs_bucket).await?);

        let session_repo = Arc::new(ProjectRedisRepository::new(redis_pool.clone()));

        let redis_data_manager = FlowProjectRedisDataManager::new(&redis_url).await?;

        let service = Arc::new(ManageEditSessionService::new(
            session_repo.clone(),
            storage.clone(),
            Arc::new(redis_data_manager),
        ));

        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

        let service_clone = service.clone();
        tokio::spawn(async move {
            if let Err(e) = service_clone.process(rx).await {
                error!("Service processing error: {}", e);
            }
        });

        Ok(AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            redis_pool,
            storage,
            session_repo,
            service,
            command_tx: tx,
        })
    }

    /// Creates a new room with the given ID.
    ///
    /// # Errors
    /// Returns `TryLockError` if the rooms mutex is poisoned or locked.
    pub fn make_room(&self, room_id: String) -> Result<(), tokio::sync::TryLockError> {
        let mut rooms = self.rooms.try_lock()?;
        rooms.insert(room_id, Room::new());
        Ok(())
    }

    /// Deletes a room with the given ID.
    ///
    /// # Errors
    /// Returns `TryLockError` if the rooms mutex is poisoned or locked.
    pub fn delete_room(&self, id: String) -> Result<(), tokio::sync::TryLockError> {
        let mut rooms = self.rooms.try_lock()?;
        rooms.remove(&id);
        Ok(())
    }

    /// Handles disconnection by cleaning up resources
    pub async fn on_disconnect(&self) {
        debug!("Handling disconnect - cleaning up rooms");
        if let Ok(mut rooms) = self.rooms.try_lock() {
            rooms.clear();
        }
    }

    /// Adds a user to a specific room
    pub async fn join(&self, room_id: &str, user_id: &str) -> Result<(), WsError> {
        let mut rooms = self.rooms.try_lock()?;
        let room = rooms
            .get_mut(room_id)
            .ok_or_else(|| WsError::RoomNotFound(room_id.to_string()))?;

        room.join(user_id.to_string()).await?;
        debug!("User {} joined room {}", user_id, room_id);
        Ok(())
    }

    /// Removes a user from a specific room
    pub async fn leave(&self, room_id: &str, user_id: &str) -> Result<(), WsError> {
        if let Ok(mut rooms) = self.rooms.try_lock() {
            if let Some(room) = rooms.get_mut(room_id) {
                room.leave(user_id.to_string()).await?;
                debug!("User {} left room {}", user_id, room_id);
                Ok(())
            } else {
                debug!("Room {} not found for user {}", room_id, user_id);
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// Broadcasts a message to all rooms
    pub async fn emit(&self, data: &str) {
        if let Ok(rooms) = self.rooms.try_lock() {
            for (room_id, room) in rooms.iter() {
                match room.broadcast(data.to_string()) {
                    Ok(_) => debug!("Message broadcast to room {}", room_id),
                    Err(e) => error!("Failed to broadcast to room {}: {:?}", room_id, e),
                }
            }
        }
    }

    /// Handles room timeout by cleaning up
    pub async fn timeout(&self) {
        debug!("Room timeout - cleaning up rooms");
        if let Ok(mut rooms) = self.rooms.try_lock() {
            rooms.clear();
        }
    }
}
