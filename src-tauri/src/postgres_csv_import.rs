use std::error::Error;
use futures_util::SinkExt;
use std::pin::pin;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio_postgres::{Client, NoTls};
use bytes::Bytes;
use csv::ReaderBuilder;
use std::fs;

#[tauri::command]
pub async fn import_csv_to_postgres(
    connection_string: String,
    path_to_file: String,
    table_name: String,
) -> Result<(), String> {
    // Connect to PostgreSQL
    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| e.to_string())?;

    // Spawn connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // First read CSV headers synchronously to determine column names and types
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(&path_to_file)
        .map_err(|e| e.to_string())?;
    
    let headers = csv_reader.headers().map_err(|e| e.to_string())?;
    let columns: Vec<(String, String)> = headers
        .iter()
        .map(|header| (header.to_string(), "text".to_string()))
        .collect();

    // Create table if it doesn't exist
    let create_table_sql = format!(
        "CREATE TABLE IF NOT EXISTS \"{}\" ({})",
        table_name,
        columns
            .iter()
            .map(|(name, typ)| format!("\"{}\" {}", name, typ))
            .collect::<Vec<_>>()
            .join(", ")
    );

    client
        .execute(&create_table_sql, &[])
        .await
        .map_err(|e| format!("Failed to create table: {}", e))?;

    // Now proceed with COPY operation
    let file = File::open(&path_to_file).await.map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    // Start COPY operation
    let mut writer = client
        .copy_in(&format!("COPY {} FROM STDIN WITH CSV HEADER", table_name))
        .await
        .map_err(|e| e.to_string())?;
    
    let mut writer = pin!(writer);
    
    let mut buffer = String::new();
    while reader
        .read_line(&mut buffer)
        .await
        .map_err(|e| e.to_string())?
        > 0
    {
        // Convert the buffer to Bytes
        let line_bytes = Bytes::from(buffer.as_bytes().to_vec());
        writer.as_mut().send(line_bytes).await.map_err(|e| e.to_string())?;
        buffer.clear();
    }

    // Complete COPY operation
    writer.as_mut().finish().await.map_err(|e| e.to_string())?;

    Ok(())
}
