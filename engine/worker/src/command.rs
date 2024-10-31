use std::{collections::HashMap, io, str::FromStr, sync::Arc};

use clap::{Arg, ArgAction, ArgMatches, Command};
use google_cloud_pubsub::client::{Client, ClientConfig};
use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::{dir::setup_job_directory, uri::Uri};
use reearth_flow_runner::runner::AsyncRunner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::{self, StorageResolver};
use reearth_flow_types::Workflow;
use tokio::runtime::Runtime;

use crate::{
    asset::download_asset, event_handler::EventHandler, factory::ALL_ACTION_FACTORIES,
    pubsub::CloudPubSub, types::metadata::Metadata,
};

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
            .ok_or(crate::errors::Error::init("No workflow provided"))?;
        let metadata_path = matches
            .remove_one::<String>("metadata_path")
            .ok_or(crate::errors::Error::init("No metadata path provided"))?;
        let metadata_path =
            Uri::from_str(metadata_path.as_str()).map_err(crate::errors::Error::init)?;
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
        let runtime =
            Arc::new(Runtime::new().map_err(crate::errors::Error::FailedToCreateTokioRuntime)?);
        runtime.block_on(self.run())
    }

    async fn run(&self) -> crate::errors::Result<()> {
        let storage_resolver = Arc::new(resolve::StorageResolver::new());
        let (workflow, state, logger_factory, event_handler) =
            self.prepare(&storage_resolver).await?;
        let handler: Arc<Box<dyn reearth_flow_runtime::event::EventHandler>> =
            Arc::new(Box::new(event_handler));
        AsyncRunner::run_with_event_handler(
            workflow,
            ALL_ACTION_FACTORIES.clone(),
            logger_factory,
            storage_resolver,
            state,
            vec![handler],
        )
        .await
        .map_err(crate::errors::Error::run)
    }

    async fn prepare(
        &self,
        storage_resolver: &Arc<StorageResolver>,
    ) -> crate::errors::Result<(
        Workflow,
        Arc<State>,
        Arc<LoggerFactory>,
        EventHandler<CloudPubSub>,
    )> {
        let json = if self.workflow == "-" {
            io::read_to_string(io::stdin()).map_err(crate::errors::Error::init)?
        } else {
            let path = Uri::from_str(self.workflow.as_str()).map_err(crate::errors::Error::init)?;
            let storage = storage_resolver
                .resolve(&path)
                .map_err(crate::errors::Error::init)?;
            let bytes = storage
                .get(path.path().as_path())
                .await
                .map_err(crate::errors::Error::FailedToDownloadWorkflow)?;
            let bytes = bytes
                .bytes()
                .await
                .map_err(crate::errors::Error::FailedToDownloadWorkflow)?;
            String::from_utf8(bytes.to_vec()).map_err(crate::errors::Error::init)?
        };
        let mut workflow = Workflow::try_from(json.as_str())
            .map_err(crate::errors::Error::failed_to_create_workflow)?;

        let storage = storage_resolver
            .resolve(&self.metadata_path)
            .map_err(crate::errors::Error::init)?;

        let meta = storage
            .get(&self.metadata_path.as_path())
            .await
            .map_err(crate::errors::Error::FailedToDownloadMetadata)?;
        let meta_json = meta
            .bytes()
            .await
            .map_err(crate::errors::Error::FailedToDownloadMetadata)?;
        let meta_json =
            String::from_utf8(meta_json.to_vec()).map_err(crate::errors::Error::init)?;
        let meta: Metadata =
            serde_json::from_str(meta_json.as_str()).map_err(crate::errors::Error::init)?;

        let job_id = meta.job_id;
        let asset_path =
            setup_job_directory("worker", "assets", job_id).map_err(crate::errors::Error::init)?;

        let artifact_path = setup_job_directory("worker", "artifacts", job_id)
            .map_err(crate::errors::Error::init)?;

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
            .map_err(crate::errors::Error::failed_to_create_workflow)?;
        workflow
            .merge_with(self.vars.clone())
            .map_err(crate::errors::Error::failed_to_create_workflow)?;

        download_asset(storage_resolver, &meta.assets, &asset_path).await?;

        let action_log_uri = setup_job_directory("worker", "action-log", job_id)
            .map_err(crate::errors::Error::init)?;
        let state_uri = setup_job_directory("worker", "feature-store", job_id)
            .map_err(crate::errors::Error::init)?;
        let state =
            Arc::new(State::new(&state_uri, storage_resolver).map_err(crate::errors::Error::init)?);

        let logger_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_uri.path()),
            action_log_uri.path(),
        ));
        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(crate::errors::Error::init)?;
        let client = Client::new(config)
            .await
            .map_err(crate::errors::Error::init)?;

        let event_handler = EventHandler::new(workflow.id, job_id, CloudPubSub::new(client));
        Ok((workflow, state, logger_factory, event_handler))
    }
}
