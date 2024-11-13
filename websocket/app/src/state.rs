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

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub redis_pool: Pool<RedisConnectionManager>,
    pub storage: Arc<ProjectStorageRepository>,
    pub session_repo: Arc<ProjectRedisRepository>,
    pub service: Arc<SessionService>,
    pub redis_url: String,
    pub command_tx: mpsc::Sender<SessionCommand>,
}

impl AppState {
    pub async fn new(redis_url: Option<String>) -> Result<Self, WsError> {
        let redis_url = redis_url.unwrap_or_else(|| {
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string())
        });

        // Initialize Redis connection pool
        let manager = RedisConnectionManager::new(&*redis_url)?;
        let redis_pool = Pool::builder().build(manager).await?;

        // Initialize storage based on feature
        #[cfg(feature = "local-storage")]
        #[allow(unused_variables)]
        let storage = Arc::new(ProjectStorageRepository::new("./local_storage".into()).await?);
        #[cfg(feature = "gcs-storage")]
        #[allow(unused_variables)]
        let storage = Arc::new(ProjectStorageRepository::new("your-gcs-bucket".into()).await?);

        let session_repo = Arc::new(ProjectRedisRepository::new(redis_pool.clone()));

        let redis_data_manager = FlowProjectRedisDataManager::new(&redis_url).await?;

        let service = Arc::new(ManageEditSessionService::new(
            session_repo.clone(),
            storage.clone(),
            Arc::new(redis_data_manager),
        ));

        let (tx, rx) = mpsc::channel(32);

        let service_clone = service.clone();
        tokio::spawn(async move {
            if let Err(e) = service_clone.process(rx).await {
                error!("Service processing error: {:?}", e);
            }
        });

        Ok(AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            redis_pool,
            storage,
            session_repo,
            service,
            redis_url,
            command_tx: tx,
        })
    }

    // Room related methods
    pub fn make_room(&self, room_id: String) -> Result<(), tokio::sync::TryLockError> {
        let mut rooms = self.rooms.try_lock()?;
        rooms.insert(room_id, Room::new());
        Ok(())
    }

    pub fn delete_room(&self, id: String) -> Result<(), tokio::sync::TryLockError> {
        let mut rooms = self.rooms.try_lock()?;
        rooms.remove(&id);
        Ok(())
    }
}
