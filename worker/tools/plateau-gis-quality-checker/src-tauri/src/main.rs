// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod handler;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![handler::get_args])
        .run(tauri::generate_context!())
        .expect("error while running plateau-gis-quality-checker");
}
