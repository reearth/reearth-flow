use std::{fs, path::PathBuf};

use clap::{Arg, ArgMatches, Command};
use schemars::schema::RootSchema;

use reearth_flow_worker::{
    errors::{Error, Result},
    types::{
        diagnostic_event::DiagnosticEvent, job_complete_event::JobCompleteEvent,
        log_stream_event::LogStreamEvent, node_status_event::NodeStatusEvent,
    },
};

pub fn build_schema_events_command() -> Command {
    Command::new("schema-events")
        .about("Generate JSON Schema files for the worker's pubsub wire event types.")
        .long_about(
            "Writes JSON Schema documents for JobCompleteEvent, LogStreamEvent, \
             NodeStatusEvent, and DiagnosticEvent — derived directly from the Rust \
             types via `schemars` — into `--dir`. Run after changing any of these \
             types so the committed schema/*.json files cannot drift from the \
             structs that actually go over the wire.",
        )
        .arg(dir_arg())
}

fn dir_arg() -> Arg {
    Arg::new("dir")
        .long("dir")
        .help("Output directory for the generated schema JSON files")
        .required(true)
        .display_order(1)
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaEventsCommand {
    dir: PathBuf,
}

impl SchemaEventsCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> Result<Self> {
        let dir = matches
            .remove_one::<String>("dir")
            .ok_or(Error::init("No output dir provided"))?;
        Ok(Self {
            dir: PathBuf::from(dir),
        })
    }

    pub fn execute(&self) -> Result<()> {
        fs::create_dir_all(&self.dir).map_err(Error::init)?;
        Self::write_schema(
            &self.dir,
            "job_complete_event.json",
            schemars::schema_for!(JobCompleteEvent),
        )?;
        Self::write_schema(
            &self.dir,
            "log_stream_event.json",
            schemars::schema_for!(LogStreamEvent),
        )?;
        Self::write_schema(
            &self.dir,
            "node_status_event.json",
            schemars::schema_for!(NodeStatusEvent),
        )?;
        Self::write_schema(
            &self.dir,
            "diagnostic_event.json",
            schemars::schema_for!(DiagnosticEvent),
        )?;
        Ok(())
    }

    fn write_schema(dir: &std::path::Path, filename: &str, schema: RootSchema) -> Result<()> {
        let mut json = serde_json::to_string_pretty(&schema).map_err(Error::init)?;
        json.push('\n');
        fs::write(dir.join(filename), json).map_err(Error::init)
    }
}
