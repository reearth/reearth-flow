use std::env;

use clap::{ArgMatches, Command};

use crate::doc_action::{build_doc_action_command, DocActionCliCommand};
use crate::dot::{build_dot_command, DotCliCommand};
use crate::run::{build_run_command, RunCliCommand};
use crate::schema_action::{build_schema_action_command, SchemaActionCliCommand};
use crate::schema_workflow::{build_schema_workflow_command, SchemaWorkflowCliCommand};

pub fn build_cli() -> Command {
    Command::new("Re:Earth Flow CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(build_run_command().display_order(1))
        .subcommand(build_dot_command().display_order(2))
        .subcommand(build_schema_action_command().display_order(3))
        .subcommand(build_schema_workflow_command().display_order(4))
        .subcommand(build_doc_action_command().display_order(5))
        .arg_required_else_help(true)
        .disable_help_subcommand(true)
        .subcommand_required(true)
}

#[derive(Debug, PartialEq)]
pub enum CliCommand {
    Run(RunCliCommand),
    Dot(DotCliCommand),
    SchemaAction(SchemaActionCliCommand),
    SchemaWorkflow(SchemaWorkflowCliCommand),
    DocAction(DocActionCliCommand),
}

impl CliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let (subcommand, submatches) = matches
            .remove_subcommand()
            .ok_or(crate::errors::Error::parse("missing subcommand"))?;
        match subcommand.as_str() {
            "run" => RunCliCommand::parse_cli_args(submatches).map(CliCommand::Run),
            "dot" => DotCliCommand::parse_cli_args(submatches).map(CliCommand::Dot),
            "schema-action" => Ok(CliCommand::SchemaAction(
                SchemaActionCliCommand::parse_cli_args(submatches)?,
            )),
            "schema-workflow" => Ok(CliCommand::SchemaWorkflow(SchemaWorkflowCliCommand)),
            "doc-action" => Ok(CliCommand::DocAction(DocActionCliCommand)),
            _ => Err(crate::errors::Error::unknown_command(subcommand)),
        }
    }

    pub fn execute(self) -> crate::Result<()> {
        match self {
            CliCommand::Run(subcommand) => subcommand.execute(),
            CliCommand::Dot(subcommand) => subcommand.execute(),
            CliCommand::SchemaAction(subcommand) => subcommand.execute(),
            CliCommand::SchemaWorkflow(subcommand) => subcommand.execute(),
            CliCommand::DocAction(subcommand) => subcommand.execute(),
        }
    }
}
