use std::{collections::HashMap, io};

use clap::{ArgMatches, Args, Command, FromArgMatches};
use reearth_flow_runtime::{dag_schemas::DagSchemas, node::SYSTEM_ACTION_FACTORY_MAPPINGS};
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;

use crate::factory::ALL_ACTION_FACTORIES;

pub fn build_dot_command() -> Command {
    DotCliCommand::augment_args(
        Command::new("dot")
            .about("Show dot graph.")
            .long_about("Show dot graph."),
    )
}

#[derive(Debug, Args, Eq, PartialEq)]
pub struct DotCliCommand {
    /// Workflow file location. Use '-' to read from stdin.
    #[arg(long, env = "REEARTH_FLOW_WORKFLOW", display_order = 1)]
    workflow: String,
}

impl DotCliCommand {
    pub fn parse_cli_args(matches: ArgMatches) -> crate::Result<Self> {
        let cmd = DotCliCommand::from_arg_matches(&matches)
            .map_err(|e| crate::errors::Error::parse(e.to_string()))?;
        Ok(cmd)
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "dot");
        let storage_resolver = resolve::StorageResolver::new();
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
