use std::{collections::HashMap, io};

use clap::{Arg, ArgMatches, Command};
use reearth_flow_runtime::{dag_schemas::DagSchemas, node::SYSTEM_ACTION_FACTORY_MAPPINGS};
use reearth_flow_runtime::schema_infer::{self, Severity};
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;

use crate::factory::ALL_ACTION_FACTORIES;

pub fn build_build_command() -> Command {
    Command::new("build")
        .visible_alias("check")
        .about("Statically validate a workflow's attribute schemas (no execution).")
        .long_about("Statically validate a workflow's attribute schemas (no execution).")
        .arg(build_cli_arg())
}

fn build_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location. Use '-' to read from stdin.")
        .env("REEARTH_FLOW_WORKFLOW")
        .required(true)
        .display_order(1)
}

#[derive(Debug, Eq, PartialEq)]
pub struct BuildCliCommand {
    workflow_path: String,
}

impl BuildCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let workflow_path = matches
            .remove_one::<String>("workflow")
            .ok_or(crate::errors::Error::init("No workflow uri provided"))?;
        Ok(BuildCliCommand { workflow_path })
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "build");
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
            DagSchemas::from_graphs(workflow.entry_graph_id, workflow.graphs, factories, None)
                .map_err(crate::errors::Error::run)?;

        let result = schema_infer::infer_and_validate(&dag)
            .map_err(|e| crate::errors::Error::run(e.to_string()))?;

        let mut error_count = 0usize;
        let mut warning_count = 0usize;
        for diagnostic in &result.diagnostics {
            let severity = match diagnostic.severity {
                Severity::Error => {
                    error_count += 1;
                    "ERROR"
                }
                Severity::Warning => {
                    warning_count += 1;
                    "WARNING"
                }
            };
            eprintln!(
                "[{severity}] {} ({}): {}",
                diagnostic.node_name, diagnostic.node_id, diagnostic.message
            );
        }
        eprintln!("{error_count} error(s), {warning_count} warning(s)");

        if result.has_errors() {
            return Err(crate::errors::Error::run(format!(
                "workflow validation failed: {error_count} error(s)"
            )));
        }
        println!("\u{2714} workflow valid");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> String {
        // CARGO_MANIFEST_DIR for the cli crate is <repo>/engine/cli.
        format!(
            "{}/../runtime/tests/fixture/workflow/schema_infer/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        )
    }

    #[test]
    fn build_valid_workflow_succeeds() {
        let cmd = BuildCliCommand {
            workflow_path: fixture_path("valid.yml"),
        };
        assert!(cmd.execute().is_ok(), "valid workflow should pass validation");
    }

    #[test]
    fn build_invalid_workflow_runs() {
        // NOTE: With the current inference implementation, sources seed `open`
        // schemas and the implemented processors preserve `open`, so a hard
        // ERROR is not reachable end-to-end from a real workflow. This fixture
        // references an attribute no upstream node produces; the validator runs
        // but emits no error because the schema reaching the consumer is open.
        // The Error path itself is covered by unit tests in
        // `reearth_flow_runtime::schema_infer` using closed-schema stub producers.
        let cmd = BuildCliCommand {
            workflow_path: fixture_path("invalid.yml"),
        };
        assert!(
            cmd.execute().is_ok(),
            "open-source seeding suppresses the reference error end-to-end"
        );
    }
}
