use std::{collections::HashMap, sync::Arc};

use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::{dir::setup_job_directory, uri::Uri};
use reearth_flow_runner::runner::Runner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve;
use reearth_flow_types::Workflow;

use crate::factory::ALL_ACTION_FACTORIES;

pub(crate) fn run_flow(
    workflow_path: String,
    params: HashMap<String, String>,
) -> Result<(), crate::errors::Error> {
    let storage_resolver = Arc::new(resolve::StorageResolver::new());
    let path = Uri::for_test(workflow_path.as_str());
    let storage = storage_resolver
        .resolve(&path)
        .map_err(crate::errors::Error::invalid_path)?;
    let bytes = storage
        .get_sync(path.path().as_path())
        .map_err(crate::errors::Error::io)?;
    let json = String::from_utf8(bytes.to_vec()).map_err(crate::errors::Error::io)?;
    let mut workflow = Workflow::try_from_str(&json);
    workflow.merge_with(params);
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
    Runner::run(
        job_id.to_string(),
        workflow,
        ALL_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver,
        state,
    )
    .map_err(crate::errors::Error::execute_failed)
}
