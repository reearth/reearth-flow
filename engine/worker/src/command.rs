use std::{collections::HashMap, io, str::FromStr, sync::Arc};

use clap::{Arg, ArgAction, ArgMatches, Command};
use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::{dir::setup_job_directory, uri::Uri};
use reearth_flow_runner::runner::AsyncRunner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::{self, StorageResolver};
use reearth_flow_types::Workflow;
use tokio::runtime::Runtime;

use crate::{asset::download_asset, factory::ALL_ACTION_FACTORIES, types::metadata::Metadata};

const WORKER_ASSET_GLOBAL_PARAMETER_VARIABLE: &str = "workerAssetPath";
const WORKER_ARTIFACT_GLOBAL_PARAMETER_VARIABLE: &str = "workerArtifactPath";

pub fn build_worker_command() -> Command {
    Command::new("worker")
        .about("Start worker.")
        .long_about("Start a worker to run a workflow.")
        .arg(workflow_arg())
        .arg(asset_arg())
        .arg(vars_arg())
}

fn workflow_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location. Use '-' to read from stdin.")
        .env("REEARTH_FLOW_WORKER_WORKFLOW")
        .required(true)
        .display_order(1)
}

fn asset_arg() -> Arg {
    Arg::new("metadata_path")
        .long("metadata-path")
        .help("Metadata path")
        .env("REEARTH_FLOW_WORKER_METADATA_PATH")
        .required(true)
        .display_order(2)
}

fn vars_arg() -> Arg {
    Arg::new("var")
        .long("var")
        .help("Workflow variables")
        .required(false)
        .action(ArgAction::Append)
        .display_order(3)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunWorkerCommand {
    workflow: String,
    metadata_path: Uri,
    vars: HashMap<String, String>,
}

impl RunWorkerCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::errors::Result<Self> {
        let workflow = matches
            .remove_one::<String>("workflow")
            .ok_or(crate::errors::WorkerError::init("No workflow provided"))?;
        let metadata_path = matches.remove_one::<String>("metadata_path").ok_or(
            crate::errors::WorkerError::init("No metadata path provided"),
        )?;
        let metadata_path =
            Uri::from_str(metadata_path.as_str()).map_err(crate::errors::WorkerError::init)?;
        let vars = matches.remove_many::<String>("var");
        let vars = if let Some(vars) = vars {
            vars.into_iter()
                .flat_map(|v| {
                    let parts: Vec<&str> = v.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            HashMap::<String, String>::new()
        };
        Ok(Self {
            workflow,
            metadata_path,
            vars,
        })
    }

    pub fn execute(&self) -> crate::errors::Result<()> {
        let runtime = Arc::new(
            Runtime::new().map_err(crate::errors::WorkerError::FailedToCreateTokioRuntime)?,
        );
        runtime.block_on(self.run())
    }

    async fn run(&self) -> crate::errors::Result<()> {
        let storage_resolver = Arc::new(resolve::StorageResolver::new());
        let (workflow, state, logger_factory) = self.prepare(&storage_resolver).await?;
        AsyncRunner::run(
            workflow,
            ALL_ACTION_FACTORIES.clone(),
            logger_factory,
            storage_resolver,
            state,
        )
        .await
        .map_err(crate::errors::WorkerError::run)
    }

    async fn prepare(
        &self,
        storage_resolver: &Arc<StorageResolver>,
    ) -> crate::errors::Result<(Workflow, Arc<State>, Arc<LoggerFactory>)> {
        let json = if self.workflow == "-" {
            io::read_to_string(io::stdin()).map_err(crate::errors::WorkerError::init)?
        } else {
            let path =
                Uri::from_str(self.workflow.as_str()).map_err(crate::errors::WorkerError::init)?;
            let storage = storage_resolver
                .resolve(&path)
                .map_err(crate::errors::WorkerError::init)?;
            let bytes = storage
                .get(path.path().as_path())
                .await
                .map_err(crate::errors::WorkerError::FailedToDownloadWorkflow)?;
            let bytes = bytes
                .bytes()
                .await
                .map_err(crate::errors::WorkerError::FailedToDownloadWorkflow)?;
            String::from_utf8(bytes.to_vec()).map_err(crate::errors::WorkerError::init)?
        };
        let mut workflow = Workflow::try_from(json.as_str())
            .map_err(crate::errors::WorkerError::failed_to_create_workflow)?;

        let storage = storage_resolver
            .resolve(&self.metadata_path)
            .map_err(crate::errors::WorkerError::init)?;

        let meta = storage
            .get(&self.metadata_path.as_path())
            .await
            .map_err(crate::errors::WorkerError::FailedToDownloadMetadata)?;
        let meta_json = meta
            .bytes()
            .await
            .map_err(crate::errors::WorkerError::FailedToDownloadMetadata)?;
        let meta_json =
            String::from_utf8(meta_json.to_vec()).map_err(crate::errors::WorkerError::init)?;
        let meta: Metadata =
            serde_json::from_str(meta_json.as_str()).map_err(crate::errors::WorkerError::init)?;

        let job_id = meta.job_id;
        let asset_path = setup_job_directory("worker", "assets", job_id)
            .map_err(crate::errors::WorkerError::init)?;

        let artifact_path = setup_job_directory("worker", "artifacts", job_id)
            .map_err(crate::errors::WorkerError::init)?;

        let mut global = HashMap::new();
        global.insert(
            WORKER_ASSET_GLOBAL_PARAMETER_VARIABLE.to_string(),
            asset_path.to_string(),
        );
        global.insert(
            WORKER_ARTIFACT_GLOBAL_PARAMETER_VARIABLE.to_string(),
            artifact_path.to_string(),
        );
        workflow
            .extend_with(global)
            .map_err(crate::errors::WorkerError::failed_to_create_workflow)?;
        workflow
            .merge_with(self.vars.clone())
            .map_err(crate::errors::WorkerError::failed_to_create_workflow)?;

        download_asset(storage_resolver, &meta.assets, &asset_path).await?;

        let action_log_uri = setup_job_directory("worker", "action-log", job_id)
            .map_err(crate::errors::WorkerError::init)?;
        let state_uri = setup_job_directory("worker", "feature-store", job_id)
            .map_err(crate::errors::WorkerError::init)?;
        let state = Arc::new(
            State::new(&state_uri, storage_resolver).map_err(crate::errors::WorkerError::init)?,
        );

        let logger_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_uri.path()),
            action_log_uri.path(),
        ));
        Ok((workflow, state, logger_factory))
    }
}
