use std::sync::Arc;

use flow_websocket_domain::{generate_id, snapshot::ObjectTenant, ProjectEditingSession};
use flow_websocket_infra::persistence::{
    local_storage::{self, LocalClient},
    project_repository::{ProjectLocalRepository, ProjectRedisRepository},
    redis::{
        flow_project_redis_data_manager::FlowProjectRedisDataManager, redis_client::RedisClient,
    },
};
use flow_websocket_services::manage_project_edit_session::ManageEditSessionService;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_client = RedisClient::new("redis://localhost:6379").await?;
    let local_storage = ProjectLocalRepository::new("./local_storage".into()).await?;
    let session_repo = ProjectRedisRepository::new(Arc::new(redis_client.clone()));

    let project_id = "project_123".to_string();
    let tenant = ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned());
    let session = ProjectEditingSession::new(project_id.clone(), tenant);

    let redis_data_manager = FlowProjectRedisDataManager::new(
        project_id.clone(),
        Arc::new(Mutex::new(session)),
        Arc::new(redis_client),
    );

    let service = ManageEditSessionService::new(
        Arc::new(session_repo),
        Arc::new(local_storage),
        Arc::new(redis_data_manager),
    );

    // Create a sample task data
    let task_data = flow_websocket_services::types::ManageProjectEditSessionTaskData {
        project_id,
        session_id: generate_id(14, "session"),
        clients_count: Some(1),
        clients_disconnected_at: None,
        last_merged_at: None,
        last_snapshot_at: None,
    };

    // Process the task
    match service.process(task_data).await {
        Ok(_) => println!("Task processed successfully"),
        Err(e) => eprintln!("Error processing task: {:?}", e),
    }

    Ok(())
}
