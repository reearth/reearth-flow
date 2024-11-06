use crate::errors::WsError;

use super::room::Room;
use flow_websocket_infra::persistence::project_repository::{
    ProjectLocalRepository, ProjectRedisRepository,
};
use flow_websocket_infra::persistence::redis::flow_project_redis_data_manager::FlowProjectRedisDataManager;
use flow_websocket_infra::persistence::redis::redis_client::RedisClient as FlowRedisClient;
use flow_websocket_services::manage_project_edit_session::ManageEditSessionService;
use flow_websocket_services::manage_project_edit_session::SessionCommand;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::error;

type SessionService = ManageEditSessionService<
    ProjectRedisRepository<FlowRedisClient>,
    ProjectLocalRepository,
    FlowProjectRedisDataManager,
>;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub redis_client: FlowRedisClient,
    pub local_storage: Arc<ProjectLocalRepository>,
    pub session_repo: Arc<ProjectRedisRepository<FlowRedisClient>>,
    pub service: Arc<SessionService>,
    pub redis_url: String,
    pub command_tx: mpsc::Sender<SessionCommand>,
}

impl AppState {
    pub async fn new(redis_url: Option<String>) -> Result<Self, WsError> {
        let redis_url = redis_url.unwrap_or_else(|| {
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string())
        });

        let redis_client = FlowRedisClient::new(&redis_url).await.unwrap();
        let redis_client = Arc::new(redis_client);

        let local_storage = Arc::new(
            ProjectLocalRepository::new("./local_storage".into())
                .await
                .unwrap(),
        );
        let session_repo = Arc::new(ProjectRedisRepository::new(redis_client.clone()));

        let redis_data_manager = FlowProjectRedisDataManager::new(&redis_url).await.unwrap();

        let service = Arc::new(ManageEditSessionService::new(
            session_repo.clone(),
            local_storage.clone(),
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
            redis_client: Arc::try_unwrap(redis_client).unwrap_or_else(|arc| (*arc).clone()),
            local_storage,
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
