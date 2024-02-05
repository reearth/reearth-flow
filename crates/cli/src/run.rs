use anyhow::anyhow;
use clap::{Arg, ArgMatches, Command};
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolver;
use reearth_flow_workflow::workflow::Workflow;
use reearth_flow_workflow_runner::dag::DagExecutor;

pub fn build_run_command() -> Command {
    Command::new("run")
        .about("Start a workflow.")
        .long_about("Start a workflow .")
        .arg(workflow_cli_arg())
}

fn workflow_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location")
        .env("REEARTH_FLOW_WORKFLOW")
        .global(true)
        .display_order(1)
}

#[derive(Debug, Eq, PartialEq)]
pub struct RunCliCommand {
    workflow_uri: Uri,
}

impl RunCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> anyhow::Result<Self> {
        let workflow_uri = matches
            .remove_one::<String>("workflow")
            .map(|uri_str| Uri::for_test(&uri_str))
            .ok_or(anyhow!("No workflow uri provided"))?;
        Ok(RunCliCommand { workflow_uri })
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        debug!(args = ?self, "run-workflow");
        let storage = resolver::resolve(&self.workflow_uri)?;
        let result = storage.get(self.workflow_uri.path().as_path()).await?;
        let content = result.bytes().await?;
        let json = String::from_utf8(content.to_vec())?;
        let workflow = Workflow::try_from_str(&json)?;
        let executor = DagExecutor::new(&workflow)?;
        executor.start().await
    }
}
