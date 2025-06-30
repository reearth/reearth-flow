mod artifact;
mod asset;
mod command;
mod errors;
mod event_handler;
mod factory;
mod logger;
mod pubsub;
mod types;

use std::env;

use command::{build_worker_command, RunWorkerCommand};

fn main() {
    let app = build_worker_command().version(env!("CARGO_PKG_VERSION"));
    let matches = app.get_matches();
    env::set_var(
        "RAYON_NUM_THREADS",
        std::cmp::min((num_cpus::get() as f64 * 1.2_f64).floor() as u64, 64)
            .to_string()
            .as_str(),
    );
    let command = match RunWorkerCommand::parse_cli_args(matches) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("Failed to parse cli args: {err:?}\n");
            std::process::exit(1);
        }
    };
    if let Err(err) = logger::setup_logging_and_tracing() {
        eprintln!("Failed to setup logging: {err}\n");
        std::process::exit(1);
    }
    let return_code: i32 = if let Err(err) = command.execute() {
        eprintln!("Command failed: {err:?}\n");
        1
    } else {
        0
    };
    std::process::exit(return_code)
}
