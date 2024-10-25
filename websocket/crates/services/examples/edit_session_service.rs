use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use flow_websocket_domain::repository::ProjectSnapshotRepository;
use flow_websocket_domain::snapshot::ProjectSnapshot;
use flow_websocket_domain::{
    generate_id,
    snapshot::{Metadata, ObjectDelete, ObjectTenant, SnapshotInfo},
    ProjectEditingSession, SnapshotData,
};

use flow_websocket_infra::persistence::{
    project_repository::{ProjectLocalRepository, ProjectRedisRepository},
    redis::{
        flow_project_redis_data_manager::FlowProjectRedisDataManager, redis_client::RedisClient,
    },
};
use flow_websocket_services::{
    manage_project_edit_session::ManageEditSessionService, types::ManageProjectEditSessionTaskData,
};
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, trace, warn};

/// # Edit Session Service Example
///
/// This example demonstrates the usage of `ManageEditSessionService` to handle
/// project editing sessions.
///
/// ## Overview
///
/// The example performs the following steps:
///
/// 1. Initializes the necessary components (Redis client, local storage, etc.)
/// 2. Creates a `ManageEditSessionService` instance
/// 3. Simulates multiple tasks in a project editing session lifecycle
///
/// ## Usage
///
/// To run this example with different log levels, use:
///
/// ```shell
/// RUST_LOG=info cargo run --example edit_session_service
/// RUST_LOG=debug,websocket=trace cargo run --example edit_session_service
/// RUST_LOG=trace cargo run --example edit_session_service
/// ```
#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting edit session service example");
    debug!("Initializing components...");

    let redis_client = RedisClient::new("redis://localhost:6379").await?;
    trace!("Redis client created");

    let local_storage = ProjectLocalRepository::new("./local_storage".into()).await?;
    trace!("Local storage initialized");

    let session_repo = ProjectRedisRepository::<RedisClient>::new(Arc::new(redis_client.clone()));
    trace!("Session repository created");

    let project_id = "project_123".to_string();
    let tenant = ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned());
    let session = ProjectEditingSession::new(project_id.clone(), tenant);
    debug!(?project_id, "Project session created");

    let redis_data_manager = FlowProjectRedisDataManager::new(
        project_id.clone(),
        session.session_id.clone(),
        Arc::new(redis_client.clone()),
    );
    trace!("Redis data manager initialized");

    let service = ManageEditSessionService::new(
        Arc::new(session_repo),
        Arc::new(local_storage),
        Arc::new(redis_data_manager),
    );
    debug!("ManageEditSessionService created");

    // Simulate multiple task processing
    match simulate_multiple_tasks(&service, &project_id).await {
        Ok(_) => info!("Multiple tasks simulation completed successfully"),
        Err(e) => error!("Error during multiple tasks simulation: {:?}", e),
    }

    info!("Edit session service example completed");
    Ok(())
}

/// Simulates multiple tasks in a project editing session lifecycle
///
/// This function demonstrates the following steps:
///
/// 1. Initializing a session
/// 2. Updating client count
/// 3. Simulating clients disconnecting
/// 4. Ending the session
///
/// Each step is separated by a simulated time passage to demonstrate
/// time-dependent behaviors.
#[instrument(skip(service))]
async fn simulate_multiple_tasks(
    service: &ManageEditSessionService<
        ProjectRedisRepository<RedisClient>,
        ProjectLocalRepository,
        FlowProjectRedisDataManager,
    >,
    project_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let session_id = generate_id(14, "session");
    debug!(?session_id, "Generated new session ID");

    // Create initial snapshot before starting session
    let initial_snapshot = ProjectSnapshot::new(
        Metadata::new(
            generate_id(14, "snap"),
            project_id.to_string(),
            Some(session_id.clone()),
            "Initial Snapshot".to_string(),
            String::new(),
        ),
        SnapshotInfo::new(
            Some("system".to_string()),
            vec!["system".to_string()],
            ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(Utc::now()),
            Some(Utc::now()),
        ),
    );

    // Create initial snapshot state
    let initial_state = SnapshotData::new(
        project_id.to_string(),
        vec![], // Empty initial state
        Some("Initial State".to_string()),
        Some("system".to_string()),
    );

    // Store initial snapshot for both project and session
    service
        .snapshot_repository
        .create_snapshot(initial_snapshot.clone())
        .await?;
    service
        .snapshot_repository
        .create_snapshot_state(initial_state.clone())
        .await?;

    // Store as latest project snapshot
    service
        .snapshot_repository
        .update_latest_snapshot(initial_snapshot.clone())
        .await?;

    // Store snapshot for session
    service
        .snapshot_repository
        .create_snapshot(initial_snapshot.clone())
        .await?;
    service
        .snapshot_repository
        .create_snapshot_state(initial_state)
        .await?;

    // Task 1: Initialize session with the same session_id
    let task_data = create_task_data(project_id, &session_id, Some(1), None);
    process_task(service, task_data, "Initialize session").await?;

    // Simulate some time passing
    warn!("Simulating time passage (1 second)");
    sleep(Duration::from_secs(1)).await;

    // Task 2: Update session with client count
    let task_data = create_task_data(project_id, &session_id, Some(2), None);
    process_task(service, task_data, "Update client count").await?;

    // Simulate more time passing
    warn!("Simulating time passage (2 seconds)");
    sleep(Duration::from_secs(2)).await;

    // Task 3: Simulate clients disconnecting
    let task_data = create_task_data(project_id, &session_id, Some(0), Some(Utc::now()));
    process_task(service, task_data, "Simulate clients disconnecting").await?;

    // Simulate time passing to trigger session end
    warn!("Simulating time passage (11 seconds)");
    sleep(Duration::from_secs(11)).await;

    // Task 4: End session
    let task_data = create_task_data(
        project_id,
        &session_id,
        Some(0),
        Some(Utc::now() - chrono::Duration::seconds(11)),
    );
    process_task(service, task_data, "End session").await?;

    Ok(())
}

/// Processes a single task using the ManageEditSessionService
///
/// This function wraps the `process` method of `ManageEditSessionService`
/// with additional logging and error handling.
///
/// ## Parameters
///
/// - `service`: The `ManageEditSessionService` instance
/// - `task_data`: The task data to be processed
/// - `task_description`: A human-readable description of the task
///
/// ## Returns
///
/// Returns `Ok(())` if the task was processed successfully, or an error if
/// the task processing failed.
#[instrument(skip(service))]
async fn process_task(
    service: &ManageEditSessionService<
        ProjectRedisRepository<RedisClient>,
        ProjectLocalRepository,
        FlowProjectRedisDataManager,
    >,
    task_data: ManageProjectEditSessionTaskData,
    task_description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(?task_description, "Processing task");
    debug!(task = ?task_data, "Task details");

    match service.process(task_data, None).await {
        Ok(_) => {
            info!(?task_description, "Task processed successfully");
            Ok(())
        }
        Err(e) => {
            error!(?task_description, error = ?e, "Error processing task");
            Err(Box::new(e))
        }
    }
}

/// Creates a ManageProjectEditSessionTaskData instance
///
/// This helper function simplifies the creation of task data for the
/// ManageEditSessionService.
///
/// ## Parameters
///
/// - `project_id`: The ID of the project
/// - `session_id`: The ID of the editing session
/// - `clients_count`: The number of connected clients (optional)
/// - `clients_disconnected_at`: The timestamp when clients disconnected (optional)
///
/// ## Returns
///
/// Returns a `ManageProjectEditSessionTaskData` instance with the specified parameters.
fn create_task_data(
    project_id: &str,
    session_id: &str,
    clients_count: Option<usize>,
    clients_disconnected_at: Option<chrono::DateTime<Utc>>,
) -> ManageProjectEditSessionTaskData {
    ManageProjectEditSessionTaskData {
        project_id: project_id.to_string(),
        session_id: session_id.to_string(),
        clients_count,
        clients_disconnected_at,
        last_merged_at: None,
        last_snapshot_at: None,
    }
}
