use std::env;

use anyhow::{bail, Context};
use clap::{ArgMatches, Command};
use tracing::Level;

use crate::run::{build_run_command, RunCliCommand};

pub fn build_cli() -> Command {
    Command::new("Re:Earth Flow")
        .subcommand(build_run_command().display_order(1))
        .arg_required_else_help(true)
        .disable_help_subcommand(true)
        .subcommand_required(true)
}

#[derive(Debug, PartialEq)]
pub enum CliCommand {
    Run(RunCliCommand),
}

impl CliCommand {
    pub fn default_log_level(&self) -> Level {
        let env_level = env::var("RUST_LOG")
            .ok()
            .and_then(|s| s.parse::<Level>().ok());
        env_level.unwrap_or(match self {
            CliCommand::Run(_) => Level::INFO,
        })
    }

    pub fn parse_cli_args(mut matches: ArgMatches) -> anyhow::Result<Self> {
        let (subcommand, submatches) = matches
            .remove_subcommand()
            .context("failed to parse command")?;
        match subcommand.as_str() {
            "run" => RunCliCommand::parse_cli_args(submatches).map(CliCommand::Run),
            _ => bail!("unknown command `{subcommand}`"),
        }
    }

    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            CliCommand::Run(subcommand) => subcommand.execute().await,
        }
    }
}
