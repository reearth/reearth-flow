use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use directories::ProjectDirs;
use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::uri::Uri;
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
    let action_log_uri = {
        let p = ProjectDirs::from("reearth", "flow", "plateau-gis-quality-checker")
            .ok_or(crate::errors::Error::setup("No action log uri provided"))?;
        let p = p
            .cache_dir()
            .to_str()
            .ok_or(crate::errors::Error::setup("Invalid action log uri"))?;
        fs::create_dir_all(
            PathBuf::default()
                .join(p)
                .join("action-log")
                .join(job_id.to_string())
                .as_path(),
        )
        .map_err(crate::errors::Error::setup)?;
        Uri::for_test(format!("file://{}", p).as_str())
    };
    let state_uri = {
        let p = ProjectDirs::from("reearth", "flow", "plateau-gis-quality-checker").unwrap();
        let p = p.cache_dir().to_str().unwrap();

        fs::create_dir_all(
            PathBuf::default()
                .join(p)
                .join("feature-store")
                .join(job_id.to_string())
                .as_path(),
        )
        .map_err(crate::errors::Error::setup)?;
        Uri::for_test(format!("file://{}", p).as_str())
    };
    let state = Arc::new(State::new(&state_uri, &storage_resolver).unwrap());

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
