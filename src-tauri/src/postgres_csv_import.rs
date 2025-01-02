use futures_util::SinkExt;
use std::pin::pin;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_postgres::NoTls;
use bytes::Bytes;
use csv::ReaderBuilder;
use std::fs;
use crate::commands::{ProgressEvent, is_cancellation_requested};
use tauri::Emitter;
use serde_json::Value;

#[tauri::command]
pub async fn import_csv_to_postgres(
    window: tauri::Window,
    connection_string: String,
    path_to_file: String,
    table_name: String,
    delimiter: String,
    linebreak: String,
    fields: Vec<Value>,
) -> Result<(), String> {
    // Debug print the new parameters
    println!("Received delimiter: {}", delimiter);
    println!("Received linebreak: {}", linebreak);
    println!("Received fields: {:?}", fields);

    // Emit initial progress event
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: 0,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "counting_rows".to_string(),
            message: Some("Starting CSV import".to_string()),
        },
    );

    // Count total rows for progress reporting
    let total_rows = fs::read_to_string(&path_to_file)
        .map_err(|e| e.to_string())?
        .lines()
        .count()
        .saturating_sub(1); // Subtract 1 for header row

    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "connecting".to_string(),
            message: Some("Connecting to PostgreSQL".to_string()),
        },
    );

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
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "creating_table".to_string(),
            message: Some("Creating table if not exists".to_string()),
        },
    );

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

    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "copying_data".to_string(),
            message: Some("Starting COPY operation".to_string()),
        },
    );

    // Now proceed with COPY operation
    let file = File::open(&path_to_file).await.map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    // Start COPY operation
    let writer = client
        .copy_in(&format!("COPY {} FROM STDIN WITH CSV HEADER", table_name))
        .await
        .map_err(|e| e.to_string())?;
    
    let mut writer = pin!(writer);
    let mut buffer = String::new();
    let mut processed_rows = 0;
    let mut last_logged = 0;
    
    while reader
        .read_line(&mut buffer)
        .await
        .map_err(|e| e.to_string())?
        > 0
    {
        // Check for cancellation
        if is_cancellation_requested() {
            // Attempt to finish the current COPY operation gracefully
            if let Err(e) = writer.as_mut().finish().await {
                eprintln!("Error finishing COPY operation during cancellation: {}", e);
            }
            
            // Emit cancellation event
            let _ = window.emit(
                "migration_progress",
                ProgressEvent {
                    total_rows,
                    processed_rows,
                    row_count: processed_rows,
                    batch_size: 0,
                    status: "cancelled".to_string(),
                    message: Some("Migration cancelled by user".to_string()),
                },
            );
            
            return Err("Migration cancelled by user".to_string());
        }

        processed_rows += 1;
        
        // Log progress every 10,000 rows
        if processed_rows - last_logged >= 10_000 {
            last_logged = processed_rows;
            let _ = window.emit(
                "migration_progress",
                ProgressEvent {
                    total_rows,
                    processed_rows,
                    row_count: processed_rows,
                    batch_size: 0,
                    status: "processing".to_string(),
                    message: Some(format!(
                        "Processed {} rows ({:.1}%)",
                        processed_rows,
                        (processed_rows as f64 / total_rows as f64) * 100.0
                    )),
                },
            );
        }

        // Convert the buffer to Bytes
        let line_bytes = Bytes::from(buffer.as_bytes().to_vec());
        writer.send(line_bytes).await.map_err(|e| e.to_string())?;
        buffer.clear();
    }

    // Complete COPY operation
    writer.as_mut().finish().await.map_err(|e| e.to_string())?;

    // Final progress event
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows,
            row_count: processed_rows,
            batch_size: 0,
            status: "complete".to_string(),
            message: Some(format!("Successfully imported {} rows", processed_rows)),
        },
    );

    Ok(())
}
