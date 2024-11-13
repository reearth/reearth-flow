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

    // Create test users
    let test_user1 = User {
        id: "user_123".to_string(),
        email: Some("test1@example.com".to_string()),
        name: Some("Test User 1".to_string()),
        tenant_id: "tenant_123".to_string(),
    };

    let test_user2 = User {
        id: "user_456".to_string(),
        email: Some("test2@example.com".to_string()),
        name: Some("Test User 2".to_string()),
        tenant_id: "tenant_123".to_string(),
    };

    debug!("Creating editing session...");
    let session = service
        .get_or_create_editing_session(project_id, test_user1.clone())
        .await?;
    info!("Created session: {:?}", session);

    // User 1's updates
    let doc1 = Doc::new();
    let text1 = doc1.get_or_insert_text("content");
    let update1 = {
        let mut txn = doc1.transact_mut();
        text1.push(&mut txn, "User 1's initial content");
        txn.encode_update_v2()
    };

    debug!("Pushing User 1's first update...");
    service
        .merge_updates(project_id, update1, Some(test_user1.id.clone()))
        .await?;

    let second_update1 = {
        let mut txn = doc1.transact_mut();
        text1.push(&mut txn, " - More from User 1");
        txn.encode_update_v2()
    };

    debug!("Pushing User 1's second update...");
    service
        .merge_updates(project_id, second_update1, Some(test_user1.id.clone()))
        .await?;

    // User 2's updates
    let doc2 = Doc::new();
    let text2 = doc2.get_or_insert_text("content");
    let update2 = {
        let mut txn = doc2.transact_mut();
        text2.push(&mut txn, "User 2's content");
        txn.encode_update_v2()
    };

    debug!("Pushing User 2's first update...");
    service
        .merge_updates(project_id, update2.clone(), Some(test_user2.id.clone()))
        .await?;

    let second_update2 = {
        let mut txn = doc2.transact_mut();
        text2.push(&mut txn, " - Additional content from User 2");
        txn.encode_update_v2()
    };

    debug!("Pushing User 2's second update...");
    service
        .merge_updates(project_id, second_update2, Some(test_user2.id.clone()))
        .await?;

    // Merge updates for User 1
    debug!("Merging updates for User 1...");
    service
        .merge_updates(project_id, update2, Some(test_user2.id.clone()))
        .await?;

    // Check state after User 1's merge
    debug!("Getting state after User 1's merge...");
    if let Some(state) = service.get_current_state(project_id).await? {
        if !state.is_empty() {
            let doc = Doc::new();
            let update = Update::decode_v2(&state).map_err(Box::new)?;
            doc.transact_mut().apply_update(update);

            let text = doc.get_or_insert_text("content");
            let content = {
                let txn = doc.transact();
                text.get_string(&txn)
            };

            info!("Content after User 1's merge: {}", content);
        }
    }

    // Check final state after both users' merges
    debug!("Getting final state after all merges...");
    if let Some(state) = service.get_current_state(project_id).await? {
        if !state.is_empty() {
            let doc = Doc::new();
            let update = Update::decode_v2(&state).map_err(Box::new)?;
            doc.transact_mut().apply_update(update);

            let text = doc.get_or_insert_text("content");
            let content = {
                let txn = doc.transact();
                text.get_string(&txn)
            };

            info!("Final content after all merges: {}", content);
            info!("Final state size: {} bytes", state.len());
        }
    }

    debug!("Ending session...");
    service.end_session(project_id.to_string(), session).await?;

    info!("Project service example completed successfully");
    Ok(())
}
