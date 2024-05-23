use colored::{Color, Colorize};
use opentelemetry::global::shutdown_tracer_provider;

use reearth_flow_cli::cli::{build_cli, CliCommand};
use reearth_flow_cli::logger;
use reearth_flow_cli::Result;

const RED_COLOR: Color = Color::TrueColor {
    r: 230,
    g: 0,
    b: 34,
};

fn main() -> Result<()> {
    let about_text = about_text();
    let app = build_cli().about(about_text).version("0.1.0");
    let matches = app.get_matches();
    let command = CliCommand::parse_cli_args(matches)?;
    logger::setup_logging_and_tracing(command.default_log_level(), true);
    let return_code: i32 = if let Err(err) = command.execute() {
        eprintln!("{} Command failed: {:?}\n", "✘".color(RED_COLOR), err);
        1
    } else {
        0
    };
    shutdown_tracer_provider();
    std::process::exit(return_code)
}

fn about_text() -> String {
    String::from("Build and run workflows to calculate and convert various data\n\n")
}
