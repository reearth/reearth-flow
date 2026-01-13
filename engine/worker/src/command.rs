use std::{collections::HashMap, io, str::FromStr, sync::Arc};

use clap::{Arg, ArgAction, ArgMatches, Command};
use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::{
    dir::setup_job_directory,
    uri::{Protocol, Uri},
};
use reearth_flow_runner::runner::AsyncRunner;
use reearth_flow_runtime::incremental::IncrementalRunConfig;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::{self, StorageResolver};
use reearth_flow_types::Workflow;
use uuid::Uuid;

use crate::{
    artifact::upload_artifact,
    asset::download_asset,
    event_handler::{EventHandler, NodeFailureHandler},
    factory::ALL_ACTION_FACTORIES,
    incremental::{prepare_incremental_artifacts, prepare_incremental_feature_store, DirCopySpec},
    logger::{enable_file_logging, set_pubsub_context, USER_FACING_LOG_HANDLER},
    pubsub::{backend::PubSubBackend, publisher::Publisher},
    types::{
        job_complete_event::{JobCompleteEvent, JobResult},
        metadata::Metadata,
    },
};

use tokio::runtime::Handle;

const WORKER_ASSET_GLOBAL_PARAMETER_VARIABLE: &str = "workerAssetPath";
const WORKER_ARTIFACT_GLOBAL_PARAMETER_VARIABLE: &str = "workerArtifactPath";

// Special UUID for workflow definition errors
const WORKFLOW_PARSE_ERROR_UUID: Uuid = Uuid::nil(); // 00000000-0000-0000-0000-000000000000

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
        .arg(previous_job_id_arg())
        .arg(start_node_id_arg())
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

fn previous_job_id_arg() -> Arg {
    Arg::new("previous_job_id")
        .long("previous-job-id")
        .help("Job ID to reuse intermediate data from")
        .required(false)
}

fn start_node_id_arg() -> Arg {
    Arg::new("start_node_id")
        .long("start-node-id")
        .help("Start node id for incremental run")
        .required(false)
}

fn merge_flow_var_env(vars: &mut HashMap<String, String>) {
    for (k, v) in std::env::vars() {
        if let Some(name) = k.strip_prefix("FLOW_VAR_") {
            if name.is_empty() {
                continue;
            }
            vars.entry(name.to_string()).or_insert(v);
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunWorkerCommand {
    workflow: String,
    metadata_path: Uri,
    vars: HashMap<String, String>,
    worker_num: usize,
    pubsub_backend: String,
    previous_job_id: Option<String>,
    start_node_id: Option<String>,
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
            .remove_one::<String>("worker_num")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(num_cpus::get());
        let pubsub_backend = matches
            .remove_one::<String>("pubsub_backend")
            .unwrap_or_else(|| "google".to_string());
        let vars = matches.remove_many::<String>("var");
        let mut vars = if let Some(vars) = vars {
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
        merge_flow_var_env(&mut vars);
        let previous_job_id = matches.remove_one::<String>("previous_job_id");
        let start_node_id = matches.remove_one::<String>("start_node_id");
        Ok(Self {
            workflow,
            metadata_path,
            vars,
            worker_num,
            pubsub_backend,
            previous_job_id,
            start_node_id,
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

        let meta = self.download_metadata(&storage_resolver).await?;
        enable_file_logging(meta.job_id)?;

        let workflow_yaml = self.download_workflow(&storage_resolver).await?;

        let (workflow_id, workflow_name) = Self::extract_workflow_info(&workflow_yaml);
        let workflow_id = workflow_id.unwrap_or(WORKFLOW_PARSE_ERROR_UUID);

        let pubsub = PubSubBackend::try_from(self.pubsub_backend.as_str())
            .await
            .map_err(crate::errors::Error::init)?;

        let handle = Handle::current();
        set_pubsub_context(pubsub.clone(), workflow_id, meta.job_id, handle)
            .map_err(crate::errors::Error::init)?;

        if let Some(name) = workflow_name {
            if let Some(handler) = USER_FACING_LOG_HANDLER.get() {
                handler.set_workflow_name(name);
            }
        }

        let mut workflow = match Workflow::try_from(workflow_yaml.as_str()) {
            Ok(w) => w,
            Err(e) => {
                if let Some(handler) = USER_FACING_LOG_HANDLER.get() {
                    handler.send_workflow_definition_error(&e);
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                self.cleanup(&meta, &storage_resolver).await?;

                return Err(crate::errors::Error::failed_to_create_workflow(e));
            }
        };

        let (ingress_state, feature_state, logger_factory, incremental_run_config) = self
            .prepare_workflow(&storage_resolver, &meta, &mut workflow)
            .await?;

        let handler: Arc<dyn reearth_flow_runtime::event::EventHandler> = match pubsub.clone() {
            PubSubBackend::Google(p) => Arc::new(EventHandler::new(workflow.id, meta.job_id, p)),
            PubSubBackend::Noop(p) => Arc::new(EventHandler::new(workflow.id, meta.job_id, p)),
        };

        let workflow_id = workflow.id;
        let node_failure_handler = Arc::new(NodeFailureHandler::new());
        let result = AsyncRunner::run_with_event_handler(
            meta.job_id,
            workflow,
            ALL_ACTION_FACTORIES.clone(),
            logger_factory,
            storage_resolver.clone(),
            ingress_state,
            feature_state,
            incremental_run_config,
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

    async fn download_metadata(
        &self,
        storage_resolver: &Arc<StorageResolver>,
    ) -> crate::errors::Result<Metadata> {
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

        Ok(meta)
    }

    async fn download_workflow(
        &self,
        storage_resolver: &Arc<StorageResolver>,
    ) -> crate::errors::Result<String> {
        let (yaml_content, base_dir) = if self.workflow == "-" {
            let content = io::read_to_string(io::stdin()).map_err(crate::errors::Error::init)?;
            (content, None)
        } else {
            let path = Uri::from_str(self.workflow.as_str()).map_err(crate::errors::Error::init)?;

            // Extract base directory for !include resolution
            let base_dir = if path.protocol() == Protocol::File {
                path.path().parent().map(|p| p.to_path_buf())
            } else {
                None
            };

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
            let content = String::from_utf8(bytes.to_vec()).map_err(crate::errors::Error::init)?;
            (content, base_dir)
        };

        // Expand !include directives if we have a base directory
        let expanded = if let Some(base) = base_dir.as_ref() {
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, Some(base))
                .map_err(crate::errors::Error::init)?
        } else {
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, None)
                .map_err(crate::errors::Error::init)?
        };

        Ok(expanded)
    }

    fn extract_workflow_info(yaml: &str) -> (Option<Uuid>, Option<String>) {
        match serde_yaml::from_str::<serde_yaml::Value>(yaml) {
            Ok(value) => {
                let id = value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());
                let name = value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                (id, name)
            }
            Err(_) => (None, None),
        }
    }

    async fn prepare_workflow(
        &self,
        storage_resolver: &Arc<StorageResolver>,
        meta: &Metadata,
        workflow: &mut Workflow,
    ) -> crate::errors::Result<(
        Arc<State>,
        Arc<State>,
        Arc<LoggerFactory>,
        Option<IncrementalRunConfig>,
    )> {
        let job_id = meta.job_id;
        let asset_path =
            setup_job_directory("workers", "assets", job_id).map_err(crate::errors::Error::init)?;

        let artifact_path = setup_job_directory("workers", "artifacts", job_id)
            .map_err(crate::errors::Error::init)?;

        let temp_artifact_path = setup_job_directory("workers", "temp-artifacts", job_id)
            .map_err(crate::errors::Error::init)?;
        let temp_artifact_root = temp_artifact_path
            .path()
            .to_str()
            .ok_or_else(|| crate::errors::Error::init("Invalid temp-artifacts dir path"))?
            .to_string();
        std::env::set_var(
            "FLOW_RUNTIME_JOB_TEMP_ARTIFACT_DIRECTORY",
            temp_artifact_root,
        );

        let mut global = HashMap::new();
        global.insert(
            WORKER_ASSET_GLOBAL_PARAMETER_VARIABLE.to_string(),
            asset_path.to_string(),
        );
        if let Some(v) = self.vars.get(WORKER_ARTIFACT_GLOBAL_PARAMETER_VARIABLE) {
            tracing::info!(
                "workerArtifactPath is provided externally. Using caller value in globals: {}",
                v
            );
            global.insert(
                WORKER_ARTIFACT_GLOBAL_PARAMETER_VARIABLE.to_string(),
                v.clone(),
            );
        } else {
            tracing::info!(
                "workerArtifactPath is not provided. Injecting job-scoped default: {}",
                artifact_path
            );
            global.insert(
                WORKER_ARTIFACT_GLOBAL_PARAMETER_VARIABLE.to_string(),
                artifact_path.to_string(),
            );
        }
        workflow
            .extend_with(global)
            .map_err(crate::errors::Error::failed_to_create_workflow)?;
        workflow
            .merge_with(self.vars.clone())
            .map_err(crate::errors::Error::failed_to_create_workflow)?;

        download_asset(storage_resolver, &meta.assets, &asset_path).await?;

        let action_log_uri = setup_job_directory("workers", "action-log", job_id)
            .map_err(crate::errors::Error::init)?;
        let feature_state_uri = setup_job_directory("workers", "feature-store", job_id)
            .map_err(crate::errors::Error::init)?;
        let feature_state = Arc::new(
            State::new(&feature_state_uri, storage_resolver).map_err(crate::errors::Error::init)?,
        );
        let ingress_state = Arc::clone(&feature_state);

        let mut incremental_run_config: Option<IncrementalRunConfig> = None;

        if let Some(prev_job_id) = &self.previous_job_id {
            tracing::info!(
                "Incremental run parameter: previous_job_id = {}",
                prev_job_id
            );
        }
        if let Some(start_node_id) = &self.start_node_id {
            tracing::info!(
                "Incremental run parameter: start_node_id = {}",
                start_node_id
            );
        }

        if let (Some(prev_job_str), Some(start_node_str)) =
            (&self.previous_job_id, &self.start_node_id)
        {
            let prev_job_id =
                uuid::Uuid::parse_str(prev_job_str).map_err(crate::errors::Error::init)?;
            let start_node_id =
                uuid::Uuid::parse_str(start_node_str).map_err(crate::errors::Error::init)?;

            let previous_feature_state = prepare_incremental_feature_store(
                "workers",
                workflow,
                job_id,
                storage_resolver.as_ref(),
                meta,
                prev_job_id,
                start_node_id,
                feature_state.as_ref(),
            )
            .await?;

            prepare_incremental_artifacts(
                "workers",
                storage_resolver.as_ref(),
                meta,
                prev_job_id,
                job_id,
                &[
                    DirCopySpec::new("artifacts", "previous-artifacts"),
                    DirCopySpec::new("temp-artifacts", "previous-temp-artifacts"),
                ],
            )
            .await?;

            previous_feature_state
                .rewrite_feature_store_file_paths_in_root_dir(prev_job_id, job_id)
                .map_err(crate::errors::Error::init)?;
            feature_state
                .rewrite_feature_store_file_paths_in_root_dir(prev_job_id, job_id)
                .map_err(crate::errors::Error::init)?;

            incremental_run_config = Some(IncrementalRunConfig {
                start_node_id,
                previous_feature_state,
            });
        } else if self.previous_job_id.is_some() || self.start_node_id.is_some() {
            tracing::info!("Incremental snapshot requires both --previous-job-id and --start-node-id. Ignoring.");
        } else {
            tracing::info!("No incremental snapshot parameters provided. Running full workflow.");
        }

        let logger_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_uri.path()),
            action_log_uri.path(),
        ));
        Ok((
            ingress_state,
            feature_state,
            logger_factory,
            incremental_run_config,
        ))
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
