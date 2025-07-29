// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{collections::HashMap, env};

use handler::QualityCheckWorkflow;
use log::{debug, LevelFilter};
use tauri_plugin_log::{LogTarget, RotationStrategy, TimezoneStrategy};

mod errors;
mod factory;
mod handler;

fn main() {
    let log_level = env::var("RUST_LOG")
        .map(|v| v.parse().unwrap_or(LevelFilter::Info))
        .unwrap_or(LevelFilter::Info);

    let tauri_loggger = tauri_plugin_log::Builder::default()
        .targets([LogTarget::Stdout, LogTarget::LogDir, LogTarget::Webview])
        .max_file_size(1_000_000) // in bytes
        .rotation_strategy(RotationStrategy::KeepOne)
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .level(log_level)
        .build();

    tauri::Builder::default()
        .plugin(tauri_loggger)
        .invoke_handler(tauri::generate_handler![
            run_flow,
            get_quality_check_workflows,
        ])
        .run(tauri::generate_context!())
        .expect("error while running plateau-gis-quality-checker");
}

#[tauri::command(async)]
pub(crate) async fn run_flow(
    workflow_id: String,
    params: HashMap<String, String>,
) -> Result<(), crate::errors::Error> {
    debug!("Running workflow: workflow id = {workflow_id:?}, params = {params:?}");

    // Execute workflow
    match handler::run_flow(workflow_id, params).await {
        Ok(_) => {
            debug!("Workflow executed successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Workflow execution failed: {e:?}");
            Err(e)
        }
    }
}

#[tauri::command]
pub(crate) fn get_quality_check_workflows() -> Vec<QualityCheckWorkflow> {
    debug!("Getting quality check workflows");
    handler::get_quality_check_workflows()
}
