use std::{collections::HashMap, io};

use clap::{Arg, ArgMatches, Command};
use reearth_flow_runtime::{dag_schemas::DagSchemas, node::SYSTEM_ACTION_FACTORY_MAPPINGS};
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;

use crate::factory::ALL_ACTION_FACTORIES;

pub fn build_dot_command() -> Command {
    Command::new("dot")
        .about("Show dot graph.")
        .long_about("Show dot graph.")
        .arg(dot_cli_arg())
}

fn dot_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location. Use '-' to read from stdin.")
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
            .ok_or(crate::errors::Error::init("No workflow uri provided"))?;
        Ok(DotCliCommand { workflow_path })
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "dot");
        let storage_resolver = resolve::StorageResolver::new();
        let json = if self.workflow_path == "-" {
            io::read_to_string(io::stdin()).map_err(crate::errors::Error::init)?
        } else {
            let path = Uri::for_test(self.workflow_path.as_str());
            let storage = storage_resolver
                .resolve(&path)
                .map_err(crate::errors::Error::init)?;
            let bytes = storage
                .get_sync(path.path().as_path())
                .map_err(crate::errors::Error::init)?;
            String::from_utf8(bytes.to_vec()).map_err(crate::errors::Error::init)?
        };
        let mut factories = HashMap::new();
        factories.extend(ALL_ACTION_FACTORIES.clone());
        factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());
        let workflow = Workflow::try_from(json.as_str()).map_err(crate::errors::Error::run)?;
        let dag =
            DagSchemas::from_graphs(workflow.entry_graph_id, workflow.graphs, factories, None);
        println!("{}", dag.to_dot());
        Ok(())
    }
}
