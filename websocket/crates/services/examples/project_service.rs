use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use flow_websocket_infra::{
    persistence::{
        project_repository::{ProjectLocalRepository, ProjectRedisRepository},
        redis::flow_project_redis_data_manager::FlowProjectRedisDataManager,
    },
    types::user::User,
};
use flow_websocket_services::project::ProjectService;
use std::sync::Arc;
use tracing::{debug, info};

///RUST_LOG=debug cargo run --example project_service

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting project service example");

    // Initialize Redis connection
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string());
    let manager = RedisConnectionManager::new(&*redis_url)?;
    let redis_pool = Pool::builder().build(manager).await?;

    // Initialize repositories and managers
    let local_storage = ProjectLocalRepository::new("./local_storage".into()).await?;
    let session_repo = ProjectRedisRepository::new(redis_pool.clone());
    let redis_data_manager = FlowProjectRedisDataManager::new(&redis_url).await?;

    // Create ProjectService instance
    let service = ProjectService::new(
        Arc::new(session_repo),
        Arc::new(local_storage),
        Arc::new(redis_data_manager),
    );

    // Example project ID
    let project_id = "test_project_123";

    // Create test user
    let test_user = User {
        id: "user_123".to_string(),
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        tenant_id: "tenant_123".to_string(),
    };

    // Demonstrate service operations
    debug!("Getting project details...");
    if let Some(project) = service.get_project(project_id).await? {
        info!("Found project: {:?}", project);
    } else {
        info!("Project not found");
    }

    debug!("Creating editing session...");
    let session = service
        .get_or_create_editing_session(project_id, test_user.clone())
        .await?;
    info!("Created session: {:?}", session);

    debug!("Listing snapshots...");
    let snapshots = service.list_all_snapshots_versions(project_id).await?;
    info!("Available snapshots: {:?}", snapshots);

    debug!("Checking allowed actions...");
    let actions = vec!["read".to_string(), "write".to_string()];
    let allowed_actions = service
        .get_project_allowed_actions(project_id, actions)
        .await?;
    info!("Allowed actions: {:?}", allowed_actions);

    debug!("Pushing update...");
    let update = b"test update".to_vec();
    service
        .push_update_to_redis_stream(project_id, update, Some(test_user.name.clone()))
        .await?;

    debug!("Getting current state...");
    if let Some(state) = service
        .get_current_state(project_id, session.session_id.as_deref())
        .await?
    {
        info!("Current state size: {} bytes", state.len());
    }

    debug!("Getting latest snapshot...");
    if let Some(snapshot) = service.get_latest_snapshot(project_id).await? {
        info!("Latest snapshot: {:?}", snapshot);
    }

    debug!("Ending session...");
    service
        .end_session("test_snapshot".to_string(), session)
        .await?;

    info!("Project service example completed successfully");
    Ok(())
}
