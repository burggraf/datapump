// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use db::connect_db;

#[tauri::command]
async fn connect(url: String) -> Result<(), String> {
    match connect_db(&url).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![connect])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
