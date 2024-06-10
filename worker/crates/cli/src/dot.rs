use std::io;

use clap::{Arg, ArgMatches, Command};
use reearth_flow_runner::executor::ACTION_MAPPINGS;
use reearth_flow_runtime::dag_schemas::DagSchemas;
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;

pub fn build_dot_command() -> Command {
    Command::new("dot")
        .about("Show dot graph.")
        .long_about("Show dot graph.")
        .arg(dot_cli_arg())
}

fn dot_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location")
        .env("REEARTH_FLOW_WORKFLOW")
        .required(true)
        .display_order(1)
}

#[derive(Debug, Eq, PartialEq)]
pub struct DotCliCommand {
    workflow_path: String,
}

impl DotCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let workflow_path = matches
            .remove_one::<String>("workflow")
            .ok_or(crate::Error::init("No workflow uri provided"))?;
        Ok(DotCliCommand { workflow_path })
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "dot");
        let storage_resolver = resolve::StorageResolver::new();
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
        let workflow = Workflow::try_from_str(&json);
        let dag = DagSchemas::from_graphs(
            workflow.entry_graph_id,
            workflow.graphs,
            ACTION_MAPPINGS.clone(),
            None,
        );
        println!("{}", dag.to_dot());
        Ok(())
    }
}
