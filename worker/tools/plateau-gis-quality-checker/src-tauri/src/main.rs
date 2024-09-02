// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::HashMap;

use log::{debug, LevelFilter};
use tauri_plugin_log::{LogTarget, RotationStrategy, TimezoneStrategy};

mod errors;
mod factory;
mod handler;

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn main() {
    let tauri_loggger = tauri_plugin_log::Builder::default()
        .targets([LogTarget::Stdout, LogTarget::LogDir, LogTarget::Webview])
        .max_file_size(1_000_000) // in bytes
        .rotation_strategy(RotationStrategy::KeepOne)
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .level(LOG_LEVEL)
        .build();

    tauri::Builder::default()
        .plugin(tauri_loggger)
        .invoke_handler(tauri::generate_handler![run_flow,])
        .run(tauri::generate_context!())
        .expect("error while running plateau-gis-quality-checker");
}

#[tauri::command]
pub(crate) fn run_flow(
    workflow_path: String,
    params: HashMap<String, String>,
) -> Result<(), crate::errors::Error> {
    debug!(
        "Running workflow: workflow path = {:?}, params = {:?}",
        workflow_path, params
    );
    handler::run_flow(workflow_path, params)
}
