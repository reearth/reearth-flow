use std::env;

use clap::{ArgMatches, Command};
use tracing::Level;

use crate::dot::{build_dot_command, DotCliCommand};
use crate::run::{build_run_command, RunCliCommand};

pub fn build_cli() -> Command {
    Command::new("Re:Earth Flow")
        .subcommand(build_run_command().display_order(1))
        .subcommand(build_dot_command().display_order(2))
        .arg_required_else_help(true)
        .disable_help_subcommand(true)
        .subcommand_required(true)
}

#[derive(Debug, PartialEq)]
pub enum CliCommand {
    Run(RunCliCommand),
    Dot(DotCliCommand),
}

impl CliCommand {
    pub fn default_log_level(&self) -> Level {
        let env_level = env::var("RUST_LOG")
            .ok()
            .and_then(|s| s.parse::<Level>().ok());
        env_level.unwrap_or(match self {
            CliCommand::Run(_) => Level::INFO,
            CliCommand::Dot(_) => Level::WARN,
        })
    }

    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let (subcommand, submatches) = matches
            .remove_subcommand()
            .ok_or(crate::Error::parse("missing subcommand"))?;
        match subcommand.as_str() {
            "run" => RunCliCommand::parse_cli_args(submatches).map(CliCommand::Run),
            "dot" => DotCliCommand::parse_cli_args(submatches).map(CliCommand::Dot),
            _ => Err(crate::Error::unknown_command(subcommand)),
        }
    }

    pub async fn execute(self) -> crate::Result<()> {
        match self {
            CliCommand::Run(subcommand) => subcommand.execute().await,
            CliCommand::Dot(subcommand) => subcommand.execute().await,
        }
    }
}
