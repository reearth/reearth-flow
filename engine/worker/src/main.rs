mod asset;
mod command;
mod errors;
mod factory;
mod logger;
mod types;

use command::{build_worker_command, RunWorkerCommand};

fn main() {
    let app = build_worker_command().version("0.1.0");
    let matches = app.get_matches();
    let command = match RunWorkerCommand::parse_cli_args(matches) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("Failed to parse cli args: {:?}\n", err);
            std::process::exit(1);
        }
    };
    logger::setup_logging_and_tracing();
    let return_code: i32 = if let Err(err) = command.execute() {
        eprintln!("Command failed: {:?}\n", err);
        1
    } else {
        0
    };
    std::process::exit(return_code)
}
