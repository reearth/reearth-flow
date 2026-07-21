mod artifact;
mod asset;
mod command;
mod event_handler;
mod factory;
mod incremental;
mod probe_schema;
mod schema_events;

use std::env;

use command::{build_worker_command, RunWorkerCommand};
use probe_schema::ProbeSchemaCommand;
use reearth_flow_worker::logger;
use schema_events::SchemaEventsCommand;

fn main() {
    let app = build_worker_command().version(env!("CARGO_PKG_VERSION"));
    let mut matches = app.get_matches();
    env::set_var(
        "RAYON_NUM_THREADS",
        std::cmp::min((num_cpus::get() as f64 * 1.2_f64).floor() as u64, 64)
            .to_string()
            .as_str(),
    );

    // No OTel guard exists yet here, so there's nothing to flush before this exit.
    let otel_guard = match logger::setup_logging_and_tracing() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to setup logging: {err}\n");
            std::process::exit(1);
        }
    };

    // `probe-schema` and `schema-events` are read-only, side-effect-free
    // subcommands (schema probe / schema codegen respectively). Everything
    // else falls through to the existing run behavior unchanged.
    //
    // Every exit below this point runs after `otel_guard` has been created, so it MUST go
    // through `shutdown_and_exit` rather than `std::process::exit` directly (see its doc).
    let return_code: i32 = match matches.remove_subcommand() {
        Some((name, sub)) if name == "probe-schema" => {
            let command = match ProbeSchemaCommand::parse_cli_args(sub) {
                Ok(command) => command,
                Err(err) => {
                    eprintln!("Failed to parse cli args: {err:?}\n");
                    shutdown_and_exit(&otel_guard, 1);
                }
            };
            if let Err(err) = command.execute() {
                eprintln!("Command failed: {err:?}\n");
                1
            } else {
                0
            }
        }
        Some((name, sub)) if name == "schema-events" => {
            let command = match SchemaEventsCommand::parse_cli_args(sub) {
                Ok(command) => command,
                Err(err) => {
                    eprintln!("Failed to parse cli args: {err:?}\n");
                    shutdown_and_exit(&otel_guard, 1);
                }
            };
            if let Err(err) = command.execute() {
                eprintln!("Command failed: {err:?}\n");
                1
            } else {
                0
            }
        }
        _ => {
            let command = match RunWorkerCommand::parse_cli_args(matches) {
                Ok(command) => command,
                Err(err) => {
                    eprintln!("Failed to parse cli args: {err:?}\n");
                    shutdown_and_exit(&otel_guard, 1);
                }
            };
            if let Err(err) = command.execute() {
                eprintln!("Command failed: {err:?}\n");
                1
            } else {
                0
            }
        }
    };
    shutdown_and_exit(&otel_guard, return_code);
}

/// Flushes any buffered OTel spans/metrics (via `OtelGuard::shutdown`) and exits with `code`.
/// `std::process::exit` skips `Drop`, so every exit after `otel_guard` is created must route
/// through here. Not covered: `panic = "abort"` aborts without unwinding, so buffered spans are
/// lost in that case.
fn shutdown_and_exit(otel_guard: &Option<logger::OtelGuard>, code: i32) -> ! {
    if let Some(guard) = otel_guard {
        guard.shutdown();
    }
    std::process::exit(code)
}
