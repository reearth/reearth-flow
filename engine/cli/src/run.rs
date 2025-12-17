use std::{collections::HashMap, io, str::FromStr, sync::Arc};

use clap::{Arg, ArgAction, ArgMatches, Command};
use reearth_flow_runner::runner::Runner;
use reearth_flow_state::State;
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::{dir::setup_job_directory, uri::Uri};
use reearth_flow_storage::resolve;

use crate::factory::ALL_ACTION_FACTORIES;
use crate::incremental::prepare_incremental_feature_store;

pub fn build_run_command() -> Command {
    Command::new("run")
        .about("Start a workflow.")
        .long_about("Start a workflow .")
        .arg(workflow_cli_arg())
        .arg(job_id_cli_arg())
        .arg(dataframe_state_cli_arg())
        .arg(action_log_cli_arg())
        .arg(vars_arg())
        .arg(previous_job_id_arg())
        .arg(start_node_id_arg())
}

fn workflow_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location. Use '-' to read from stdin.")
        .env("REEARTH_FLOW_WORKFLOW")
        .required(true)
        .display_order(1)
}

fn job_id_cli_arg() -> Arg {
    Arg::new("job_id")
        .long("job-id")
        .help("Job id")
        .env("REEARTH_FLOW_JOB_ID")
        .required(false)
        .display_order(2)
}

fn dataframe_state_cli_arg() -> Arg {
    Arg::new("dataframe_state")
        .long("dataframe-state")
        .help("Dataframe state location")
        .env("REEARTH_FLOW_DATAFRAME_STATE")
        .required(false)
        .display_order(3)
}

fn action_log_cli_arg() -> Arg {
    Arg::new("action_log")
        .long("action-log")
        .help("Action log location")
        .env("REEARTH_FLOW_ACTION_LOG")
        .required(false)
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

#[derive(Debug, Eq, PartialEq)]
pub struct RunCliCommand {
    workflow_path: String,
    job_id: Option<String>,
    dataframe_state_uri: Option<String>,
    action_log_uri: Option<String>,
    vars: HashMap<String, String>,
    previous_job_id: Option<String>,
    start_node_id: Option<String>,
}

impl RunCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let workflow_path = matches
            .remove_one::<String>("workflow")
            .ok_or(crate::errors::Error::init("No workflow uri provided"))?;
        let job_id = matches.remove_one::<String>("job_id");
        let dataframe_state_uri = matches.remove_one::<String>("dataframe_state");
        let action_log_uri = matches.remove_one::<String>("action_log");
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
        let previous_job_id = matches.remove_one::<String>("previous_job_id");
        let start_node_id = matches.remove_one::<String>("start_node_id");
        Ok(RunCliCommand {
            workflow_path,
            job_id,
            dataframe_state_uri,
            action_log_uri,
            vars,
            previous_job_id,
            start_node_id,
        })
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "run-workflow");
        let storage_resolver = Arc::new(resolve::StorageResolver::new());
        let (yaml_content, base_dir) = if self.workflow_path == "-" {
            let content = io::read_to_string(io::stdin()).map_err(crate::errors::Error::init)?;
            (content, None)
        } else {
            let path = Uri::for_test(self.workflow_path.as_str());

            // Extract base directory for !include resolution
            let base_dir = path.path().parent().map(|p| p.to_path_buf());

            let storage = storage_resolver
                .resolve(&path)
                .map_err(crate::errors::Error::init)?;
            let bytes = storage
                .get_sync(path.path().as_path())
                .map_err(crate::errors::Error::init)?;
            let content = String::from_utf8(bytes.to_vec()).map_err(crate::errors::Error::init)?;
            (content, base_dir)
        };

        // Expand !include directives
        let json = if let Some(base) = base_dir.as_ref() {
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, Some(base))
                .map_err(crate::errors::Error::init)?
        } else {
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, None)
                .map_err(crate::errors::Error::init)?
        };

        let mut workflow = Workflow::try_from(json.as_str()).map_err(crate::errors::Error::init)?;
        workflow
            .merge_with(self.vars.clone())
            .map_err(crate::errors::Error::init)?;
        let job_id = match &self.job_id {
            Some(job_id) => {
                uuid::Uuid::from_str(job_id.as_str()).map_err(crate::errors::Error::init)?
            }
            None => uuid::Uuid::new_v4(),
        };
        let action_log_uri = match &self.action_log_uri {
            Some(uri) => Uri::from_str(uri).map_err(crate::errors::Error::init)?,
            None => setup_job_directory("engine", "action-log", job_id)
                .map_err(crate::errors::Error::init)?,
        };
        let feature_state_uri = setup_job_directory("engine", "feature-store", job_id)
            .map_err(crate::errors::Error::init)?;
        let feature_state = Arc::new(
            State::new(&feature_state_uri, &storage_resolver)
                .map_err(crate::errors::Error::init)?,
        );
        let ingress_state = Arc::clone(&feature_state);

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

            prepare_incremental_feature_store(
                &workflow,
                job_id,
                &storage_resolver,
                prev_job_id,
                start_node_id,
            )?;
        } else if self.previous_job_id.is_some() || self.start_node_id.is_some() {
            tracing::info!("Incremental snapshot requires both --previous-job-id and --start-node-id. Ignoring.");
        } else {
            tracing::info!("No incremental snapshot parameters provided. Running full workflow.");
        }

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
            ingress_state,
            feature_state,
        )
        .map_err(|e| crate::errors::Error::Run(format!("Failed to run workflow: {e}")))
    }
}
