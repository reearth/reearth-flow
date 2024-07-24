use std::{collections::HashMap, fs, io, path::Path, str::FromStr, sync::Arc};

use clap::{Arg, ArgAction, ArgMatches, Command};
use directories::ProjectDirs;
use reearth_flow_runner::runner::Runner;
use reearth_flow_state::State;
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;

use crate::factory::BUILTIN_ACTION_FACTORIES;

pub fn build_run_command() -> Command {
    Command::new("run")
        .about("Start a workflow.")
        .long_about("Start a workflow .")
        .arg(workflow_cli_arg())
        .arg(job_id_cli_arg())
        .arg(dataframe_state_cli_arg())
        .arg(action_log_cli_arg())
        .arg(vars_arg())
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

#[derive(Debug, Eq, PartialEq)]
pub struct RunCliCommand {
    workflow_path: String,
    job_id: Option<String>,
    dataframe_state_uri: Option<String>,
    action_log_uri: Option<String>,
    vars: HashMap<String, String>,
}

impl RunCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let workflow_path = matches
            .remove_one::<String>("workflow")
            .ok_or(crate::Error::init("No workflow uri provided"))?;
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
        Ok(RunCliCommand {
            workflow_path,
            job_id,
            dataframe_state_uri,
            action_log_uri,
            vars,
        })
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "run-workflow");
        let storage_resolver = Arc::new(resolve::StorageResolver::new());
        let json = if self.workflow_path == "-" {
            io::read_to_string(io::stdin()).map_err(crate::Error::init)?
        } else {
            let path = Uri::for_test(self.workflow_path.as_str());
            let storage = storage_resolver
                .resolve(&path)
                .map_err(crate::Error::init)?;
            let bytes = storage
                .get_sync(path.path().as_path())
                .map_err(crate::Error::init)?;
            String::from_utf8(bytes.to_vec()).map_err(crate::Error::init)?
        };
        let mut workflow = Workflow::try_from_str(&json);
        workflow.merge_with(self.vars.clone());
        let job_id = match &self.job_id {
            Some(job_id) => uuid::Uuid::from_str(job_id.as_str()).map_err(crate::Error::init)?,
            None => uuid::Uuid::new_v4(),
        };
        let action_log_uri = match &self.action_log_uri {
            Some(uri) => Uri::from_str(uri).map_err(crate::Error::init)?,
            None => {
                let p = ProjectDirs::from("reearth", "flow", "worker")
                    .ok_or(crate::Error::init("No action log uri provided"))?;
                let p = p
                    .cache_dir()
                    .to_str()
                    .ok_or(crate::Error::init("Invalid action log uri"))?;
                let p = format!("{}/action-log/{}", p, job_id);
                fs::create_dir_all(Path::new(p.as_str())).map_err(crate::Error::init)?;
                Uri::for_test(format!("file://{}", p).as_str())
            }
        };
        let state_uri = {
            let p = ProjectDirs::from("reearth", "flow", "worker").unwrap();
            let p = p.cache_dir().to_str().unwrap();
            let p = format!("{}/feature-store/{}", p, job_id);
            let _ = fs::create_dir_all(Path::new(p.as_str()));
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
            BUILTIN_ACTION_FACTORIES.clone(),
            logger_factory,
            storage_resolver,
            state,
        );
        Ok(())
    }
}
