mod factory;
use clap::Parser;
use log::info;
use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_action_log;
use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::dir::setup_job_directory;
use reearth_flow_common::uri::Uri;
use reearth_flow_state::State;
use reearth_flow_storage::resolve;
use reearth_flow_types::Workflow;

use factory::ALL_ACTION_FACTORIES;

// TODO: This is a placeholder for the actual implementation of the worker.
// This is a placeholder for the actual implementation of the worker.
// I don't know whether to use sync or async, so I've implemented both. Once you've decided which to use,

#[derive(Parser, Debug)]
struct Args {
    url: String,
}

#[cfg(not(feature = "feature-async"))]
fn main() {
    use reearth_flow_runner::runner::Runner;
    // TODO: Prepare Process
    // TODO: Please make sure to handle errors properly in the 'expect' section.

    let args = Args::parse();

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let url = &args.url;
    let rc = reqwest::blocking::get(url).expect("failed to download from url");
    let src = &rc.text().expect("failed to get text");

    let job_id: Option<String> = None; // TODO: Read from ??
    let action_log_uri: Option<String> = None; // TODO: Read from ??
    let workflow = Workflow::try_from_str(src).expect("invalid yaml file");

    let storage_resolver = Arc::new(resolve::StorageResolver::new());
    let job_id = match job_id {
        Some(job_id) => uuid::Uuid::from_str(job_id.as_str()).expect("Invalid job id"),
        None => uuid::Uuid::new_v4(),
    };
    let action_log_uri = match action_log_uri {
        Some(uri) => Uri::from_str(&uri).expect("Invalid action log uri"),
        None => setup_job_directory("worker", "action-log", job_id)
            .expect("Failed to setup job directory"),
    };
    info!("{:?}", action_log_uri);
    let state_uri = setup_job_directory("worker", "feature-store", job_id)
        .expect("Failed to setup job directory");
    let state =
        Arc::new(State::new(&state_uri, &storage_resolver).expect("Failed to create state"));

    let logger_factory = Arc::new(LoggerFactory::new(
        create_root_logger(action_log_uri.path()),
        action_log_uri.path(),
    ));
    Runner::run(
        workflow,
        ALL_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver,
        state,
    )
    .expect("Failed to run workflow");

    // TODO: Clean up Process
}

#[cfg(feature = "feature-async")]
#[tokio::main]
async fn main() {
    use reearth_flow_runner::runner::AsyncRunner;
    // TODO: Prepare Process
    // TODO: Please make sure to handle errors properly in the 'expect' section.
    let yaml = "${yamlcode}"; // TODO: Read from ??
    let job_id: Option<String> = None; // TODO: Read from ??
    let action_log_uri: Option<String> = None; // TODO: Read from ??
    let workflow = Workflow::try_from_str(yaml);

    let storage_resolver = Arc::new(resolve::StorageResolver::new());
    let job_id = match job_id {
        Some(job_id) => uuid::Uuid::from_str(job_id.as_str()).expect("Invalid job id"),
        None => uuid::Uuid::new_v4(),
    };
    let action_log_uri = match action_log_uri {
        Some(uri) => Uri::from_str(&uri).expect("Invalid action log uri"),
        None => setup_job_directory("worker", "action-log", job_id)
            .expect("Failed to setup job directory"),
    };
    let state_uri = setup_job_directory("worker", "feature-store", job_id)
        .expect("Failed to setup job directory");
    let state =
        Arc::new(State::new(&state_uri, &storage_resolver).expect("Failed to create state"));

    let logger_factory = Arc::new(LoggerFactory::new(
        create_root_logger(action_log_uri.path()),
        action_log_uri.path(),
    ));
    AsyncRunner::run(
        workflow,
        ALL_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver,
        state,
    )
    .await
    .expect("Failed to run workflow");

    // TODO: Clean up Process
}
