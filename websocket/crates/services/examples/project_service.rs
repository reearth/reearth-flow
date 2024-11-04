// use flow_websocket_domain::generate_id;
// use flow_websocket_domain::repository::RedisDataManager;
// use flow_websocket_infra::persistence::{
//     project_repository::{ProjectLocalRepository, ProjectRedisRepository},
//     redis::{
//         flow_project_redis_data_manager::FlowProjectRedisDataManager, redis_client::RedisClient,
//     },
// };
// use flow_websocket_services::project::ProjectService;
// use std::sync::Arc;
// use tracing::{debug, error, info, instrument, trace, warn};

// /// # Project Service Example
// ///
// /// This example demonstrates the usage of `ProjectService` to handle
// /// project-related operations.
// ///
// /// ## Overview
// ///
// /// The example performs the following steps:
// ///
// /// 1. Initializes the necessary components (Redis client, local storage, etc.)
// /// 2. Creates a `ProjectService` instance
// /// 3. Simulates various project-related operations
// ///
// /// ## Usage
// ///
// /// To run this example with different log levels, use:
// ///
// /// ```shell
// /// RUST_LOG=info cargo run --example project_service
// /// RUST_LOG=debug,websocket=trace cargo run --example project_service
// /// RUST_LOG=trace cargo run --example project_service
// /// ```
// #[tokio::main]
// #[instrument]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Initialize tracing
//     tracing_subscriber::fmt()
//         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//         .init();

//     info!("Starting project service example");
//     debug!("Initializing components...");

//     let redis_client = RedisClient::new("redis://localhost:6379").await?;
//     trace!("Redis client created");

//     let local_storage = ProjectLocalRepository::new("./local_storage".into()).await?;
//     trace!("Local storage initialized");

//     let session_repo = ProjectRedisRepository::<RedisClient>::new(Arc::new(redis_client.clone()));
//     trace!("Session repository created");

//     let project_id = generate_id(14, "project");
//     debug!(?project_id, "Project ID generated");

//     let session_id = generate_id(14, "session");
//     debug!(?session_id, "Session ID generated");

//     let redis_data_manager = FlowProjectRedisDataManager::new(
//         project_id.clone(),
//         Some(session_id),
//         Arc::new(redis_client.clone()),
//     );
//     trace!("Redis data manager initialized");

//     let service = ProjectService::new(
//         Arc::new(session_repo),
//         Arc::new(local_storage),
//         Arc::new(redis_data_manager),
//     );
//     debug!("ProjectService created");

//     // Simulate project operations
//     match simulate_project_operations(&service, &project_id).await {
//         Ok(_) => info!("Project operations simulation completed successfully"),
//         Err(e) => error!("Error during project operations simulation: {:?}", e),
//     }

//     info!("Project service example completed");
//     Ok(())
// }

// /// Simulates various project operations using the ProjectService
// ///
// /// This function demonstrates the following operations:
// ///
// /// 1. Creating a project
// /// 2. Getting project details
// /// 3. Creating and retrieving an editing session
// /// 4. Getting project allowed actions
// /// 5. Pushing an update to the project
// /// 6. Getting the current project state
// ///
// #[instrument(skip(service))]
// async fn simulate_project_operations(
//     service: &ProjectService<
//         ProjectRedisRepository<RedisClient>,
//         ProjectLocalRepository,
//         FlowProjectRedisDataManager,
//     >,
//     project_id: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // Get project details (
//     match service.get_project(project_id).await {
//         Ok(retrieved_project) => info!(?retrieved_project, "Retrieved project details"),
//         Err(e) => {
//             error!("Failed to retrieve project: {:?}", e);
//             return Err(Box::new(e));
//         }
//     }

//     // Create and retrieve an editing session
//     let session = match service
//         .get_or_create_editing_session(project_id, None, None)
//         .await
//     {
//         Ok(session) => {
//             info!(?session, "Editing session created/retrieved");
//             session
//         }
//         Err(e) => {
//             error!("Failed to create or retrieve editing session: {:?}", e);
//             return Err(Box::new(e));
//         }
//     };

//     // Check if session ID is set
//     if session.session_id.is_none() {
//         error!("Session ID is not set after get_or_create_editing_session");
//         return Err("Session ID not set".into());
//     }

//     // Get project allowed actions
//     let actions = vec![
//         "read".to_string(),
//         "write".to_string(),
//         "delete".to_string(),
//     ];
//     let allowed_actions = service
//         .get_project_allowed_actions(project_id, actions)
//         .await?;
//     info!(?allowed_actions, "Retrieved project allowed actions");

//     // Push an update to the project
//     let update = vec![1, 2, 3, 4, 5]; // Example update data
//     match service
//         .push_update(update.clone(), Some("user1".to_string()))
//         .await
//     {
//         Ok(_) => info!(?update, "Update pushed to the project"),
//         Err(e) => warn!("Failed to push update: {:?}", e),
//     }

//     // Get current project state
//     match service.get_current_state().await {
//         Ok(current_state) => {
//             info!(?current_state, "Retrieved current project state");
//             debug!("Current state structure: {:#?}", current_state);
//         }
//         Err(e) => {
//             error!("Failed to get current state: {:?}", e);

//             if let Some(response_start) = e.to_string().find("response was [") {
//                 let response_str = &e.to_string()[response_start..];
//                 debug!(
//                     "Redis response structure: {}",
//                     response_str.trim_matches(|c| c == '[' || c == ']')
//                 );

//                 warn!("Redis response contains a nested structure with state updates");
//                 debug!("Expected format should match: [key, [[timestamp, [field, value, field, value]], ...]]");
//             }

//             return Err(Box::new(e));
//         }
//     }

//     Ok(())
// }

fn main() {
    todo!()
}
