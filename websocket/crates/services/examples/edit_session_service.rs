use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use flow_websocket_domain::{generate_id, user::User};
use flow_websocket_infra::persistence::{
    project_repository::{ProjectLocalRepository, ProjectRedisRepository},
    redis::flow_project_redis_data_manager::FlowProjectRedisDataManager,
};
use flow_websocket_services::manage_project_edit_session::{
    ManageEditSessionService, SessionCommand,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting edit session service example");

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string());

    // Initialize Redis connection pool
    let manager = RedisConnectionManager::new(&*redis_url)?;
    let redis_pool = Pool::builder().build(manager).await?;

    // Initialize components
    let local_storage = ProjectLocalRepository::new("./local_storage".into()).await?;
    let session_repo = ProjectRedisRepository::new(redis_pool.clone());
    let redis_data_manager = FlowProjectRedisDataManager::new(&redis_url).await?;

    let project_id = "project_123".to_string();

    // Create service
    let service = ManageEditSessionService::new(
        Arc::new(session_repo),
        Arc::new(local_storage),
        Arc::new(redis_data_manager),
    );

    // Create channel for commands
    let (tx, rx) = mpsc::channel(32);
    let service_clone = service.clone();

    // Spawn service processing task
    let process_handle = tokio::spawn(async move {
        if let Err(e) = service_clone.process(rx).await {
            error!("Service processing error: {:?}", e);
        }
    });

    // Create test user
    let test_user = User {
        id: generate_id!("user"),
        email: "test.user@example.com".to_string(),
        name: "Test User".to_string(),
        tenant_id: generate_id!("tenant"),
    };

    // Simulate session lifecycle
    debug!("Starting session simulation");

    tx.send(SessionCommand::AddTask {
        project_id: project_id.clone(),
    })
    .await?;

    // Start session
    tx.send(SessionCommand::Start {
        project_id: project_id.clone(),
        user: test_user.clone(),
    })
    .await?;

    // Check status
    tx.send(SessionCommand::CheckStatus {
        project_id: project_id.clone(),
    })
    .await?;

    // List snapshots
    tx.send(SessionCommand::ListAllSnapshotsVersions {
        project_id: project_id.clone(),
    })
    .await?;

    // Push update
    tx.send(SessionCommand::PushUpdate {
        project_id: project_id.clone(),
        update: vec![1, 2, 3],
        updated_by: Some(test_user.name.clone()),
    })
    .await?;

    // End session
    tx.send(SessionCommand::End {
        project_id: project_id.clone(),
        user: test_user.clone(),
    })
    .await?;

    // // Remove task
    // tx.send(SessionCommand::RemoveTask {
    //     project_id: project_id.clone(),
    // })
    // .await?;

    // // Complete session
    // tx.send(SessionCommand::Complete {
    //     project_id,
    //     user: test_user,
    // })
    // .await?;

    // Drop sender to terminate service
    drop(tx);

    // Wait for service to complete
    process_handle.await?;

    info!("Edit session service example completed");
    Ok(())
}
