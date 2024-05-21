use clap::{Arg, ArgMatches, Command};
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;
use reearth_flow_workflow_runner::dag::dag_impl::Dag;
use reearth_flow_workflow_runner::types::graph::Workflow;

pub fn build_dot_command() -> Command {
    Command::new("dot")
        .about("Show dot graph.")
        .long_about("Show dot graph.")
        .arg(workflow_cli_arg())
}

fn workflow_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location")
        .env("REEARTH_FLOW_WORKFLOW")
        .required(true)
        .display_order(1)
}

#[derive(Debug, Eq, PartialEq)]
pub struct DotCliCommand {
    workflow_uri: Uri,
}

impl DotCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let workflow_uri = matches
            .remove_one::<String>("workflow")
            .map(|uri_str| Uri::for_test(&uri_str))
            .ok_or(crate::Error::init("No workflow uri provided"))?;
        Ok(DotCliCommand { workflow_uri })
    }

    pub async fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "dot");
        let storage_resolver = resolve::StorageResolver::new();
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
        for graph in workflow.graphs.iter() {
            let dag = Dag::from_graph(graph).map_err(crate::Error::init)?;
            println!("{}", dag.to_dot());
        }
        Ok(())
    }
}
