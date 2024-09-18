use std::{collections::HashMap, sync::Arc, vec};

use once_cell::sync::Lazy;
use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::dir::setup_job_directory;
use reearth_flow_runner::runner::AsyncRunner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve;
use reearth_flow_types::Workflow;
use rust_embed::Embed;

use crate::factory::ALL_ACTION_FACTORIES;

#[derive(Embed)]
#[folder = "embed/workflows/"]
struct WorkflowAsset;

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct QualityCheckWorkflow {
    id: String,
    name: String,
}

pub(crate) static QUALITY_CHECK_WORKFLOWS: Lazy<Vec<QualityCheckWorkflow>> = Lazy::new(|| {
    vec![QualityCheckWorkflow {
        id: "common".to_string(),
        name: "共通".to_string(),
    }]
});

pub(crate) async fn run_flow(
    workflow_id: String,
    params: HashMap<String, String>,
) -> Result<(), crate::errors::Error> {
    let bytes = WorkflowAsset::get(format!("{}.yml", workflow_id).as_str()).ok_or(
        crate::errors::Error::invalid_workflow_id(format!("Workflow not found: {}", workflow_id)),
    )?;
    let json = String::from_utf8(bytes.data.iter().cloned().collect())
        .map_err(crate::errors::Error::io)?;
    let mut workflow = Workflow::try_from_str(&json);
    workflow.merge_with(params);
    let storage_resolver = Arc::new(resolve::StorageResolver::new());
    let job_id = uuid::Uuid::new_v4();
    let action_log_uri = setup_job_directory("plateau-gis-quality-checker", "action-log", job_id)
        .map_err(crate::errors::Error::setup)?;
    let state_uri = setup_job_directory("plateau-gis-quality-checker", "feature-store", job_id)
        .map_err(crate::errors::Error::setup)?;
    let state =
        Arc::new(State::new(&state_uri, &storage_resolver).map_err(crate::errors::Error::setup)?);

    let logger_factory = Arc::new(LoggerFactory::new(
        create_root_logger(action_log_uri.path()),
        action_log_uri.path(),
    ));
    AsyncRunner::run(
        job_id.to_string(),
        workflow,
        ALL_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver,
        state,
    )
    .await
    .map_err(crate::errors::Error::execute_failed)
}

pub(crate) fn get_quality_check_workflows() -> Vec<QualityCheckWorkflow> {
    QUALITY_CHECK_WORKFLOWS.clone()
}
