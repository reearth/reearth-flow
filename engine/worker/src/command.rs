use std::{collections::HashMap, io, str::FromStr, sync::Arc};

use clap::{Arg, ArgAction, ArgMatches, Command};
use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::{dir::setup_job_directory, uri::Uri};
use reearth_flow_runner::runner::AsyncRunner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::{self, StorageResolver};
use reearth_flow_types::Workflow;

use crate::{
    artifact::upload_artifact,
    asset::download_asset,
    event_handler::{EventHandler, NodeFailureHandler},
    factory::ALL_ACTION_FACTORIES,
    logger::{enable_file_logging, set_pubsub_context},
    pubsub::{backend::PubSubBackend, publisher::Publisher},
    types::{
        job_complete_event::{JobCompleteEvent, JobResult},
        metadata::Metadata,
    },
};

use tokio::runtime::Handle;

const WORKER_ASSET_GLOBAL_PARAMETER_VARIABLE: &str = "workerAssetPath";
const WORKER_ARTIFACT_GLOBAL_PARAMETER_VARIABLE: &str = "workerArtifactPath";

pub fn build_worker_command() -> Command {
    Command::new("Re:Earth Flow Worker")
        .about("Start flow worker.")
        .long_about("Start a worker to run a workflow.")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(workflow_arg())
        .arg(asset_arg())
        .arg(worker_num_arg())
        .arg(pubsub_backend_arg())
        .arg(vars_arg())
}

fn workflow_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location. Use '-' to read from stdin.")
        .env("FLOW_WORKER_WORKFLOW")
        .required(true)
        .display_order(1)
}

fn asset_arg() -> Arg {
    Arg::new("metadata_path")
        .long("metadata-path")
        .help("Metadata path")
        .env("FLOW_WORKER_METADATA_PATH")
        .required(true)
        .display_order(2)
}

fn worker_num_arg() -> Arg {
    Arg::new("worker_num")
        .long("worker-num")
        .help("Number of workers")
        .env("FLOW_WORKER_WORKER_NUM")
        .required(false)
        .display_order(3)
}

fn pubsub_backend_arg() -> Arg {
    Arg::new("pubsub_backend")
        .long("pubsub-backend")
        .help("PubSub backend")
        .env("FLOW_WORKER_PUBSUB_BACKEND")
        .required(false)
        .default_value("google")
        .display_order(4)
}

fn vars_arg() -> Arg {
    Arg::new("var")
        .long("var")
        .help("Workflow variables")
        .required(false)
        .action(ArgAction::Append)
        .display_order(5)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunWorkerCommand {
    workflow: String,
    metadata_path: Uri,
    vars: HashMap<String, String>,
    worker_num: usize,
    pubsub_backend: String,
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
        let worker_num = matches
            .remove_one::<usize>("worker_num")
            .unwrap_or(num_cpus::get());
        let pubsub_backend = matches
            .remove_one::<String>("pubsub_backend")
            .unwrap_or_else(|| "google".to_string());
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
            worker_num,
            pubsub_backend,
        })
    }

    pub fn execute(&self) -> crate::errors::Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.worker_num)
            .enable_all()
            .build()
            .map_err(crate::errors::Error::FailedToCreateTokioRuntime)?;
        runtime.block_on(self.run())
    }

    async fn run(&self) -> crate::errors::Result<()> {
        tracing::info!("Starting worker");
        let storage_resolver = Arc::new(resolve::StorageResolver::new());
        let (workflow, state, logger_factory, meta) = self.prepare(&storage_resolver).await?;
        enable_file_logging(meta.job_id)?;

        let pubsub = PubSubBackend::try_from(self.pubsub_backend.as_str())
            .await
            .map_err(crate::errors::Error::init)?;

        let handle = Handle::current();

        set_pubsub_context(pubsub.clone(), workflow.id, meta.job_id, handle)
            .map_err(crate::errors::Error::init)?;

        let handler: Arc<dyn reearth_flow_runtime::event::EventHandler> = match &pubsub {
            PubSubBackend::Google(p) => {
                Arc::new(EventHandler::new(workflow.id, meta.job_id, p.clone()))
            }
            PubSubBackend::Noop(p) => {
                Arc::new(EventHandler::new(workflow.id, meta.job_id, p.clone()))
            }
        };

        let workflow_id = workflow.id;
        let node_failure_handler = Arc::new(NodeFailureHandler::new());
        let result = AsyncRunner::run_with_event_handler(
            meta.job_id,
            workflow,
            ALL_ACTION_FACTORIES.clone(),
            logger_factory,
            storage_resolver.clone(),
            state,
            vec![handler, node_failure_handler.clone()],
        )
        .await;
        let job_result = match result {
            Ok(_) => {
                if node_failure_handler.all_success() {
                    JobResult::Success
                } else {
                    tracing::error!("Failed nodes: {:?}", node_failure_handler.failed_nodes());
                    JobResult::Failed
                }
            }
            Err(_) => JobResult::Failed,
        };
        self.cleanup(&meta, &storage_resolver).await?;
        match &pubsub {
            PubSubBackend::Google(p) => p
                .publish(JobCompleteEvent::new(
                    workflow_id,
                    meta.job_id,
                    job_result.clone(),
                ))
                .await
                .map_err(crate::errors::Error::run),
            PubSubBackend::Noop(p) => p
                .publish(JobCompleteEvent::new(
                    workflow_id,
                    meta.job_id,
                    job_result.clone(),
                ))
                .await
                .map_err(|e| crate::errors::Error::run(format!("{e:?}"))),
        }?;
        tracing::info!(
            "Job completed with workflow_id: {:?}, job_id: {:?} result: {:?}",
            workflow_id,
            meta.job_id,
            job_result
        );
        Ok(())
    }

    async fn prepare(
        &self,
        storage_resolver: &Arc<StorageResolver>,
    ) -> crate::errors::Result<(Workflow, Arc<State>, Arc<LoggerFactory>, Metadata)> {
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
            setup_job_directory("workers", "assets", job_id).map_err(crate::errors::Error::init)?;

        let artifact_path = setup_job_directory("workers", "artifacts", job_id)
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

        let action_log_uri = setup_job_directory("workers", "action-log", job_id)
            .map_err(crate::errors::Error::init)?;
        let state_uri = setup_job_directory("workers", "feature-store", job_id)
            .map_err(crate::errors::Error::init)?;
        let state =
            Arc::new(State::new(&state_uri, storage_resolver).map_err(crate::errors::Error::init)?);

        let logger_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_uri.path()),
            action_log_uri.path(),
        ));
        Ok((workflow, state, logger_factory, meta))
    }

    async fn cleanup(
        &self,
        meta: &Metadata,
        storage_resolver: &Arc<StorageResolver>,
    ) -> crate::errors::Result<()> {
        upload_artifact(storage_resolver, meta).await?;
        Ok(())
    }
}
