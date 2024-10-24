use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use flow_websocket_domain::{generate_id, snapshot::ObjectTenant, ProjectEditingSession};
use flow_websocket_infra::persistence::{
    project_repository::{ProjectLocalRepository, ProjectRedisRepository},
    redis::{
        flow_project_redis_data_manager::FlowProjectRedisDataManager, redis_client::RedisClient,
    },
};
use flow_websocket_services::{
    manage_project_edit_session::ManageEditSessionService, types::ManageProjectEditSessionTaskData,
};
use tokio::sync::Mutex;
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
    // ... existing task simulation code ...

    // Add demonstrations of individual functionalities
    demonstrate_update_client_count(service, project_id).await?;
    demonstrate_merge_updates(service, project_id).await?;
    demonstrate_snapshot_creation(service, project_id).await?;
    demonstrate_session_ending(service, project_id).await?;
    demonstrate_job_completion(service, project_id).await?;

    Ok(())
}

/// Demonstrates the update_client_count functionality
#[instrument(skip(service))]
async fn demonstrate_update_client_count(
    service: &ManageEditSessionService<
        ProjectRedisRepository<RedisClient>,
        ProjectLocalRepository,
        FlowProjectRedisDataManager,
    >,
    project_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Demonstrating update_client_count functionality");

    let mut task_data = create_task_data(project_id, "session_123", Some(1), None);

    // Initial client count update
    let count = service.update_client_count(&mut task_data).await?;
    debug!(client_count = count, "Initial client count");

    // Simulate clients disconnecting
    task_data.clients_count = Some(0);
    let count = service.update_client_count(&mut task_data).await?;
    debug!(
        client_count = count,
        disconnected_at = ?task_data.clients_disconnected_at,
        "Clients disconnected"
    );

    Ok(())
}

/// Demonstrates the merge_updates functionality
#[instrument(skip(service))]
async fn demonstrate_merge_updates(
    service: &ManageEditSessionService<
        ProjectRedisRepository<RedisClient>,
        ProjectLocalRepository,
        FlowProjectRedisDataManager,
    >,
    project_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Demonstrating merge_updates functionality");

    let mut session = ProjectEditingSession::new(
        project_id.to_string(),
        ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
    );
    session.session_setup_complete = true;
    session.session_id = Some("session_123".to_string());

    let mut task_data = create_task_data(project_id, "session_123", None, None);

    service.merge_updates(&mut session, &mut task_data).await?;
    debug!(last_merged_at = ?task_data.last_merged_at, "Updates merged");

    Ok(())
}

/// Demonstrates the snapshot creation functionality
#[instrument(skip(service))]
async fn demonstrate_snapshot_creation(
    service: &ManageEditSessionService<
        ProjectRedisRepository<RedisClient>,
        ProjectLocalRepository,
        FlowProjectRedisDataManager,
    >,
    project_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Demonstrating snapshot creation functionality");

    let mut session = ProjectEditingSession::new(
        project_id.to_string(),
        ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
    );
    session.session_setup_complete = true;
    session.session_id = Some("session_123".to_string());

    // Test create_snapshot_if_required
    let mut task_data = create_task_data(project_id, "session_123", None, None);
    task_data.last_snapshot_at = Some(Utc::now() - chrono::Duration::minutes(6));

    service
        .create_snapshot_if_required(&mut session, &mut task_data)
        .await?;
    debug!(last_snapshot_at = ?task_data.last_snapshot_at, "Snapshot created if required");

    // Test direct snapshot creation
    service.create_snapshot(&mut session, Utc::now()).await?;
    debug!("Direct snapshot created");

    Ok(())
}

/// Demonstrates the session ending functionality
#[instrument(skip(service))]
async fn demonstrate_session_ending(
    service: &ManageEditSessionService<
        ProjectRedisRepository<RedisClient>,
        ProjectLocalRepository,
        FlowProjectRedisDataManager,
    >,
    project_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Demonstrating session ending functionality");

    let mut session = ProjectEditingSession::new(
        project_id.to_string(),
        ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
    );
    session.session_setup_complete = true;
    session.session_id = Some("session_123".to_string());

    let task_data = create_task_data(
        project_id,
        "session_123",
        Some(0),
        Some(Utc::now() - chrono::Duration::seconds(11)),
    );

    let ended = service
        .end_editing_session_if_conditions_met(&mut session, &task_data, 0)
        .await?;
    debug!(session_ended = ended, "Session end check completed");

    Ok(())
}

/// Demonstrates the job completion functionality
#[instrument(skip(service))]
async fn demonstrate_job_completion(
    service: &ManageEditSessionService<
        ProjectRedisRepository<RedisClient>,
        ProjectLocalRepository,
        FlowProjectRedisDataManager,
    >,
    project_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Demonstrating job completion functionality");

    let mut session = ProjectEditingSession::new(
        project_id.to_string(),
        ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
    );
    session.session_setup_complete = true;
    session.session_id = Some("session_123".to_string());

    let task_data = create_task_data(project_id, "session_123", None, None);

    service
        .complete_job_if_met_requirements(&session, &task_data)
        .await?;
    debug!("Job completion check completed");

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

    match service.process(task_data).await {
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
