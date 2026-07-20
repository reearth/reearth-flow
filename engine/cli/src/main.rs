mod cli;
mod doc_action;
mod dot;
mod errors;
mod factory;
mod incremental;
mod logger;
mod probe_schema;
mod run;
mod scaffold_i18n;
mod schema_action;
mod schema_error_codes;
mod schema_workflow;
mod utils;

use std::env;

use colored::{Color, Colorize};

use crate::cli::{build_cli, CliCommand};
use crate::errors::Result;

const RED_COLOR: Color = Color::TrueColor {
    r: 230,
    g: 0,
    b: 34,
};

fn main() -> Result<()> {
    let about_text = about_text();
    let app = build_cli()
        .about(about_text)
        .version(env!("CARGO_PKG_VERSION"));
    let matches = app.get_matches();
    let command = CliCommand::parse_cli_args(matches)?;
    env::set_var(
        "RAYON_NUM_THREADS",
        std::cmp::min((num_cpus::get() as f64 * 1.2_f64).floor() as u64, 64)
            .to_string()
            .as_str(),
    );
    let otel_guard = logger::setup_logging_and_tracing()?;
    let return_code: i32 = if let Err(err) = command.execute() {
        eprintln!("{} Command failed: {:?}\n", "✘".color(RED_COLOR), err);
        1
    } else {
        0
    };
    // std::process::exit below skips Drop, so any OTel provider guard
    // must be flushed explicitly here before it would otherwise be lost.
    if let Some(guard) = otel_guard {
        guard.shutdown();
    }
    std::process::exit(return_code)
}

fn about_text() -> String {
    String::from("Build and run workflows to calculate and convert various data\n\n")
}
