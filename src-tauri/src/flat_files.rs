use std::fs::OpenOptions;
use std::io::Write;
use std::path;

#[tauri::command]
pub async fn get_real_path(file_path: String) -> Result<String, String> {
    let absolute_path = path::absolute(file_path).map_err(|e| e.to_string())?;
    Ok(absolute_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn append_to_file(file_path: String, text: String) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| e.to_string())?;

    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}
