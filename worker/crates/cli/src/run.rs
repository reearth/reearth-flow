use std::{fs, path::Path, str::FromStr, sync::Arc};

use clap::{Arg, ArgMatches, Command};
use directories::ProjectDirs;
use tracing::debug;

use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::uri::Uri;
use reearth_flow_state::State;
use reearth_flow_storage::resolve;
use reearth_flow_workflow::{id::Id, workflow::Workflow};
use reearth_flow_workflow_runner::dag::DagExecutor;

pub fn build_run_command() -> Command {
    Command::new("run")
        .about("Start a workflow.")
        .long_about("Start a workflow .")
        .arg(workflow_cli_arg())
        .arg(job_id_cli_arg())
        .arg(dataframe_state_cli_arg())
        .arg(action_log_cli_arg())
}

fn workflow_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location")
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

#[derive(Debug, Eq, PartialEq)]
pub struct RunCliCommand {
    workflow_uri: Uri,
    job_id: Option<String>,
    dataframe_state_uri: Option<String>,
    action_log_uri: Option<String>,
}

impl RunCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let workflow_uri = matches
            .remove_one::<String>("workflow")
            .map(|uri_str| Uri::for_test(&uri_str))
            .ok_or(crate::Error::init("No workflow uri provided"))?;
        let job_id = matches.remove_one::<String>("job_id");
        let dataframe_state_uri = matches.remove_one::<String>("dataframe_state");
        let action_log_uri = matches.remove_one::<String>("action_log");
        Ok(RunCliCommand {
            workflow_uri,
            job_id,
            dataframe_state_uri,
            action_log_uri,
        })
    }

    pub async fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "run-workflow");
        let storage_resolver = Arc::new(resolve::StorageResolver::new());
        let storage = storage_resolver
            .resolve(&self.workflow_uri)
            .map_err(crate::Error::init)?;
        let result = storage
            .get(self.workflow_uri.path().as_path())
            .await
            .map_err(crate::Error::init)?;
        let content = result.bytes().await.map_err(crate::Error::init)?;
        let json = String::from_utf8(content.to_vec()).map_err(crate::Error::init)?;
        let workflow = Workflow::try_from_str(&json).map_err(crate::Error::init)?;
        let job_id = match &self.job_id {
            Some(job_id) => Id::from_str(job_id.as_str()).map_err(crate::Error::init)?,
            None => Id::new_v4(),
        };
        let dataframe_state_uri = match &self.dataframe_state_uri {
            Some(uri) => Uri::from_str(uri).map_err(crate::Error::init)?,
            None => {
                let p = ProjectDirs::from("reearth", "flow", "worker")
                    .ok_or(crate::Error::init("No dataframe state uri provided"))?;
                let p = p
                    .data_dir()
                    .to_str()
                    .ok_or(crate::Error::init("Invalid dataframe state uri"))?;
                let p = format!("{}/dataframe/{}", p, job_id);
                fs::create_dir_all(Path::new(p.as_str())).map_err(crate::Error::init)?;
                Uri::for_test(format!("file://{}", p).as_str())
            }
        };
        let action_log_uri = match &self.action_log_uri {
            Some(uri) => Uri::from_str(uri).map_err(crate::Error::init)?,
            None => {
                let p = ProjectDirs::from("reearth", "flow", "worker")
                    .ok_or(crate::Error::init("No dataframe state uri provided"))?;
                let p = p
                    .data_dir()
                    .to_str()
                    .ok_or(crate::Error::init("Invalid dataframe state uri"))?;
                let p = format!("{}/action-log/{}", p, job_id);
                fs::create_dir_all(Path::new(p.as_str())).map_err(crate::Error::init)?;
                Uri::for_test(format!("file://{}", p).as_str())
            }
        };
        let state = Arc::new(
            State::new(&dataframe_state_uri, &storage_resolver).map_err(crate::Error::init)?,
        );
        let log_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_uri.path()),
            action_log_uri.path(),
        ));
        let executor = DagExecutor::new(job_id, &workflow, storage_resolver, state, log_factory)
            .map_err(crate::Error::init)?;
        executor.start().await.map_err(crate::Error::run)?;
        Ok(())
    }
}
