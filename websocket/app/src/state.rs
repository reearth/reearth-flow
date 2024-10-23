use super::errors::Result;
use super::room::Room;
use flow_websocket_infra::persistence::project_repository::{
    ProjectLocalRepository, ProjectRedisRepository,
};
use flow_websocket_infra::persistence::redis::flow_project_redis_data_manager::FlowProjectRedisDataManager;
use flow_websocket_infra::persistence::redis::redis_client::RedisClient as FlowRedisClient;
use flow_websocket_services::manage_project_edit_session::ManageEditSessionService;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive()]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub redis_client: FlowRedisClient,
    pub edit_session_service: Arc<
        ManageEditSessionService<
            ProjectRedisRepository<FlowRedisClient>,
            ProjectLocalRepository,
            FlowProjectRedisDataManager,
        >,
    >,
}

impl AppState {
    pub async fn new(redis_url: Option<String>) -> Result<Self> {
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

        let session_id = String::new();            "default".to_string(),
            Some(session_id.clone()),
            redis_client.clone(),
        ));

        Ok(AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            redis_client: Arc::try_unwrap(redis_client).unwrap_or_else(|arc| (*arc).clone()),
            edit_session_service: Arc::new(ManageEditSessionService::new(
                session_repo,
                local_storage,
                redis_data_manager,
            )),
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
