use std::{collections::HashMap, io, str::FromStr, sync::Arc};

use clap::{ArgMatches, Args, Command, FromArgMatches};
use reearth_flow_runner::runner::Runner;
use reearth_flow_state::State;
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::{dir::setup_job_directory, uri::Uri};
use reearth_flow_storage::resolve;

use crate::factory::ALL_ACTION_FACTORIES;

pub fn build_run_command() -> Command {
    RunCliCommand::augment_args(
        Command::new("run")
            .about("Start a workflow.")
            .long_about("Start a workflow."),
    )
}

#[derive(Debug, Args, Eq, PartialEq)]
pub struct RunCliCommand {
    /// Workflow file location. Use '-' to read from stdin.
    #[arg(long, env = "REEARTH_FLOW_WORKFLOW", display_order = 1)]
    workflow: String,

    /// Job id
    #[arg(long = "job-id", env = "REEARTH_FLOW_JOB_ID", display_order = 2)]
    job_id: Option<String>,

    /// Dataframe state location
    #[arg(
        long = "dataframe-state",
        env = "REEARTH_FLOW_DATAFRAME_STATE",
        display_order = 3
    )]
    dataframe_state: Option<String>,

    /// Action log location
    #[arg(
        long = "action-log",
        env = "REEARTH_FLOW_ACTION_LOG",
        display_order = 4
    )]
    action_log: Option<String>,

    /// Workflow variables (format: KEY=VALUE)
    #[arg(long = "var", action = clap::ArgAction::Append, display_order = 5, value_parser = parse_key_value)]
    vars: Vec<(String, String)>,

    /// Runtime Working Directory
    #[arg(
        long = "working-dir",
        env = "REEARTH_FLOW_WORKING_DIRECTORY",
        display_order = 6
    )]
    working_dir: Option<String>,

    /// Disable action log
    #[arg(
        long = "action-log-disable",
        env = "FLOW_RUNTIME_ACTION_LOG_DISABLE",
        display_order = 7
    )]
    action_log_disable: Option<bool>,

    /// Channel buffer size for worker threads
    #[arg(
        long = "channel-buffer-size",
        env = "FLOW_RUNTIME_CHANNEL_BUFFER_SIZE",
        display_order = 8
    )]
    channel_buffer_size: Option<usize>,

    /// Event hub channel capacity
    #[arg(
        long = "event-hub-capacity",
        env = "FLOW_RUNTIME_EVENT_HUB_CAPACITY",
        display_order = 9
    )]
    event_hub_capacity: Option<usize>,

    /// Worker thread pool size
    #[arg(
        long = "thread-pool-size",
        env = "FLOW_RUNTIME_THREAD_POOL_SIZE",
        display_order = 10
    )]
    thread_pool_size: Option<usize>,

    /// Feature flush threshold for sink nodes
    #[arg(
        long = "feature-flush-threshold",
        env = "FLOW_RUNTIME_FEATURE_FLUSH_THRESHOLD",
        display_order = 11
    )]
    feature_flush_threshold: Option<usize>,

    /// Async worker number (Tokio threads)
    #[arg(
        long = "async-worker-num",
        env = "FLOW_RUNTIME_ASYNC_WORKER_NUM",
        display_order = 12
    )]
    async_worker_num: Option<usize>,

    /// Disable feature writer (export to feature store)
    #[arg(
        long = "feature-writer-disable",
        env = "FLOW_RUNTIME_FEATURE_WRITER_DISABLE",
        display_order = 13
    )]
    feature_writer_disable: Option<bool>,

    /// Slow action threshold in milliseconds
    #[arg(
        long = "slow-action-threshold-ms",
        env = "FLOW_RUNTIME_SLOW_ACTION_THRESHOLD_MS",
        display_order = 14
    )]
    slow_action_threshold_ms: Option<u64>,

    /// Node status propagation delay in milliseconds
    #[arg(
        long = "node-status-propagation-delay-ms",
        env = "FLOW_RUNTIME_NODE_STATUS_PROPAGATION_DELAY_MS",
        display_order = 15
    )]
    node_status_propagation_delay_ms: Option<u64>,
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(format!("Invalid key=value pair: '{s}'"))
    }
}

impl RunCliCommand {
    pub fn parse_cli_args(matches: ArgMatches) -> crate::Result<Self> {
        let cmd = RunCliCommand::from_arg_matches(&matches)
            .map_err(|e| crate::errors::Error::parse(e.to_string()))?;
        Ok(cmd)
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "run-workflow");
        let storage_resolver = Arc::new(resolve::StorageResolver::new());
        let json = if self.workflow == "-" {
            io::read_to_string(io::stdin()).map_err(crate::errors::Error::init)?
        } else {
            let path = Uri::for_test(self.workflow.as_str());
            let storage = storage_resolver
                .resolve(&path)
                .map_err(crate::errors::Error::init)?;
            let bytes = storage
                .get_sync(path.path().as_path())
                .map_err(crate::errors::Error::init)?;
            String::from_utf8(bytes.to_vec()).map_err(crate::errors::Error::init)?
        };
        let mut workflow = Workflow::try_from(json.as_str()).map_err(crate::errors::Error::init)?;
        let vars_map: HashMap<String, String> = self.vars.iter().cloned().collect();
        workflow
            .merge_with(vars_map)
            .map_err(crate::errors::Error::init)?;
        let job_id = match &self.job_id {
            Some(job_id) => {
                uuid::Uuid::from_str(job_id.as_str()).map_err(crate::errors::Error::init)?
            }
            None => uuid::Uuid::new_v4(),
        };
        let action_log_uri = match &self.action_log {
            Some(uri) => Uri::from_str(uri).map_err(crate::errors::Error::init)?,
            None => setup_job_directory("engine", "action-log", job_id)
                .map_err(crate::errors::Error::init)?,
        };
        let state_uri = setup_job_directory("engine", "feature-store", job_id)
            .map_err(crate::errors::Error::init)?;
        let state = Arc::new(
            State::new(&state_uri, &storage_resolver).map_err(crate::errors::Error::init)?,
        );

        let logger_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_uri.path()),
            action_log_uri.path(),
        ));
        Runner::run(
            job_id,
            workflow,
            ALL_ACTION_FACTORIES.clone(),
            logger_factory,
            storage_resolver,
            state,
        )
        .map_err(|e| crate::errors::Error::Run(format!("Failed to run workflow: {e}")))
    }
}
