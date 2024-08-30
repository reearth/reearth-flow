#[tauri::command]
pub fn get_args() -> Result<Vec<String>, ()> {
  let system_args: Vec<String> = std::env::args().collect();
  Ok(system_args)
}
