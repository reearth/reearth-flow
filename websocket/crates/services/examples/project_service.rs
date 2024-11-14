use bb8::Pool;
use bb8_redis::RedisConnectionManager;

#[cfg(feature = "gcs-storage")]
use flow_websocket_infra::persistence::ProjectGcsRepository;
#[cfg(feature = "local-storage")]
use flow_websocket_infra::persistence::ProjectLocalRepository;

use flow_websocket_infra::{
    persistence::{
        project_repository::ProjectRedisRepository,
        redis::flow_project_redis_data_manager::FlowProjectRedisDataManager,
    },
    types::user::User,
};
use flow_websocket_services::project::ProjectService;
use std::sync::Arc;
use tracing::{debug, info};
use yrs::{updates::decoder::Decode, Doc, GetString, Text, Transact, Update};

///RUST_LOG=debug cargo run --example project_service
/// RUST_LOG=debug cargo run --example project_service  --features local-storage

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

    // Initialize repositories and managers based on feature
    #[cfg(feature = "local-storage")]
    #[allow(unused_variables)]
    let storage = ProjectLocalRepository::new("./local_storage".into()).await?;
    #[cfg(feature = "gcs-storage")]
    #[allow(unused_variables)]
    let storage = ProjectGcsRepository::new("your-gcs-bucket".to_string()).await?;

    let session_repo = ProjectRedisRepository::new(redis_pool.clone());
    let redis_data_manager = FlowProjectRedisDataManager::new(&redis_url).await?;

    // Create ProjectService instance
    let service = ProjectService::new(
        Arc::new(session_repo),
        Arc::new(storage),
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

    // Create a Yjs document with initial content
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    let update = {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "Initial content");
        txn.encode_update_v2()
    };

    debug!("Pushing initial update...");
    service
        .push_update_to_redis_stream(project_id, update, Some(test_user.name.clone()))
        .await?;

    // Create another update
    let second_update = {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, " - More content");
        txn.encode_update_v2()
    };

    debug!("Pushing second update...");
    service
        .push_update_to_redis_stream(project_id, second_update, Some(test_user.name.clone()))
        .await?;

    debug!("Getting current state...");
    if let Some(state) = service
        .get_current_state(project_id, session.session_id.as_deref())
        .await?
    {
        if !state.is_empty() {
            debug!("---------------------");
            debug!("state: {:?}", state);
            debug!("------------");

            // Create a new doc to apply the state
            let doc = Doc::new();
            let update = Update::decode_v2(&state).map_err(Box::new)?;
            doc.transact_mut().apply_update(update);

            let text = doc.get_or_insert_text("content");
            let content = {
                let txn = doc.transact();
                text.get_string(&txn)
            };

            info!("Current document content: {}", content);
            info!("Current state size: {} bytes", state.len());
        } else {
            debug!("Received empty state update, skipping...");
        }
    }

    debug!("Getting latest snapshot...");
    if let Some(snapshot) = service.get_latest_snapshot(project_id).await? {
        info!("Latest snapshot: {:?}", snapshot);
    }

    debug!("Ending session...");
    debug!("session: {:?}", session.session_id);
    service.end_session(project_id.to_string(), session).await?;

    info!("Project service example completed successfully");
    Ok(())
}
