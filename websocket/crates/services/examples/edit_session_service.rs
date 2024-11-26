// use bb8::Pool;
// use bb8_redis::RedisConnectionManager;
// #[cfg(feature = "gcs-storage")]
// use flow_websocket_infra::persistence::ProjectGcsRepository;
// #[cfg(feature = "local-storage")]
// use flow_websocket_infra::persistence::ProjectLocalRepository;
// use flow_websocket_infra::{
//     generate_id,
//     persistence::{
//         project_repository::ProjectRedisRepository,
//         redis::flow_project_redis_data_manager::FlowProjectRedisDataManager,
//     },
//     types::user::User,
// };
// use flow_websocket_services::manage_project_edit_session::{
//     ManageEditSessionService, SessionCommand,
// };
// use std::sync::Arc;
// use tokio::sync::broadcast;
// use tracing::info;
// use yrs::{Doc, Text, Transact};

// ///export REDIS_URL="redis://default:my_redis_password@localhost:6379/0"
// ///RUST_LOG=debug cargo run --example edit_session_service  --features local-storage
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Initialize tracing
//     tracing_subscriber::fmt()
//         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//         .init();

//     info!("Starting edit session service example");

//     let redis_url =
//         std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string());
//     // let redis_url =
//     //     std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://:my_redis_password@localhost:6379/0".to_string());

//     // Initialize Redis connection pool
//     let manager = RedisConnectionManager::new(&*redis_url)?;
//     let redis_pool = Pool::builder().build(manager).await?;
//     // Initialize storage
//     #[cfg(feature = "local-storage")]
//     #[allow(unused_variables)]
//     let storage = ProjectLocalRepository::new("./local_storage".into()).await?;
//     #[cfg(feature = "gcs-storage")]
//     #[allow(unused_variables)]
//     let storage = ProjectGcsRepository::new("your-gcs-bucket".to_string()).await?;

//     let session_repo = ProjectRedisRepository::new(redis_pool.clone());
//     let redis_data_manager = FlowProjectRedisDataManager::new(&redis_url).await?;

//     // Create service
//     let service = ManageEditSessionService::new(
//         Arc::new(session_repo),
//         Arc::new(storage.clone()),
//         Arc::new(redis_data_manager),
//         Arc::new(storage.clone()),
//         Arc::new(storage),
//     );

//     let project_id = "project_123".to_string();

//     // Create channel for commands
//     let (tx, rx) = broadcast::channel(32);
//     let service_clone = service.clone();

//     // Spawn service processing task
//     let process_handle = tokio::spawn(async move { service_clone.process(rx).await });

//     // Create test user
//     let test_user = User {
//         id: generate_id!("user"),
//         email: Some("test.user@example.com".to_string()),
//         name: Some("Test User".to_string()),
//         tenant_id: generate_id!("tenant"),
//     };

//     // Simulate session lifecycle
//     info!("Starting session simulation");

//     tx.send(SessionCommand::AddTask {
//         project_id: project_id.clone(),
//     })?;

//     info!("Starting session");

//     // Start session
//     tx.send(SessionCommand::Start {
//         project_id: project_id.clone(),
//         user: test_user.clone(),
//     })?;

//     info!("Checking status");

//     // Check status
//     tx.send(SessionCommand::CheckStatus {
//         project_id: project_id.clone(),
//     })?;

//     info!("Listing all snapshots versions");

//     // List snapshots
//     tx.send(SessionCommand::ListAllSnapshotsVersions {
//         project_id: project_id.clone(),
//     })?;

//     info!("Creating Y.js document and update");

//     // Create Y.js document and update
//     let doc = Doc::new();
//     let text = doc.get_or_insert_text("test");
//     let yjs_update = {
//         let mut txn = doc.transact_mut();
//         text.push(&mut txn, "Hello, YJS!");
//         txn.encode_update_v2()
//     };

//     info!("Pushing update with Y.js data");

//     // Push update with Y.js data
//     tx.send(SessionCommand::MergeUpdates {
//         project_id: project_id.clone(),
//         data: yjs_update,
//         updated_by: Some(test_user.id.clone()),
//     })?;

//     info!("Creating second update");

//     // Create second update
//     let yjs_update2 = {
//         let mut txn = doc.transact_mut();
//         text.push(&mut txn, " More text!");
//         txn.encode_update_v2()
//     };

//     info!("Pushing second update");

//     // Push second update
//     tx.send(SessionCommand::MergeUpdates {
//         project_id: project_id.clone(),
//         data: yjs_update2,
//         updated_by: Some(test_user.id.clone()),
//     })?;

//     info!("Checking status again after merge");

//     // Check status again after merge
//     tx.send(SessionCommand::CheckStatus {
//         project_id: project_id.clone(),
//     })?;

//     info!("Ending session");

//     // End session
//     tx.send(SessionCommand::End {
//         project_id: project_id.clone(),
//         user: test_user.clone(),
//     })?;

//     info!("Removing task");

//     // Remove task
//     tx.send(SessionCommand::RemoveTask {
//         project_id: project_id.clone(),
//     })?;

//     // // Complete session
//     // tx.send(SessionCommand::Complete {
//     //     project_id,
//     //     user: test_user,
//     // })
//     // .await?;

//     // Drop sender to terminate service
//     drop(tx);

//     // Wait for service to complete
//     process_handle.await?;

//     info!("Edit session service example completed");
//     Ok(())
// }

fn main() {}
