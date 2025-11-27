// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;

use log::LevelFilter;
use tauri_plugin_log::{LogTarget, RotationStrategy, TimezoneStrategy};

mod commands;
mod error;

use commands::AppState;
use reearth_flow_analyzer::default_reports_dir;

fn main() {
    let log_level = env::var("RUST_LOG")
        .map(|v| v.parse().unwrap_or(LevelFilter::Info))
        .unwrap_or(LevelFilter::Info);

    let tauri_logger = tauri_plugin_log::Builder::default()
        .targets([LogTarget::Stdout, LogTarget::LogDir, LogTarget::Webview])
        .max_file_size(1_000_000)
        .rotation_strategy(RotationStrategy::KeepOne)
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .level(log_level)
        .build();

    // Get the default reports directory
    let reports_dir = default_reports_dir();

    tauri::Builder::default()
        .manage(AppState::new(reports_dir))
        .plugin(tauri_logger)
        .invoke_handler(tauri::generate_handler![
            commands::list_reports,
            commands::load_report,
            commands::get_node_memory_data,
            commands::get_node_queue_data,
            commands::set_reports_directory,
            commands::get_reports_directory,
            commands::get_report_nodes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running reearth-flow-analyzer");
}
