use futures_util::SinkExt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_postgres::NoTls;
use bytes::BytesMut;
use crate::commands::{ProgressEvent, is_cancellation_requested};
use crate::postgres_writer::{start_copy, finish_copy};
use tauri::Emitter;
use serde_json::Value;
use chrono;
use csv::{ReaderBuilder, Trim};
use std::str;
use serde::{Serialize, Deserialize};
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Deserialize)]
struct Field {
    name: String,
    field_type: String,
}

impl Field {
    fn to_postgres_type(&self) -> String {
        match self.field_type.as_str() {
            "integer" => "NUMERIC".to_string(),
            "number" => "NUMERIC".to_string(),
            "date" => "DATE".to_string(),
            _ => "TEXT".to_string()
        }
    }

    fn validate_value(&self, value: &str) -> bool {
        if value.trim().is_empty() {
            return true; // Allow empty values
        }

        match self.field_type.as_str() {
            "integer" => value.parse::<f64>().is_ok(),
            "number" => value.parse::<f64>().is_ok(),
            "date" => {
                chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok() || // YYYY-MM-DD
                chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok() || // YYYY-MM-DD HH:MM:SS
                chrono::NaiveDate::parse_from_str(value, "%m/%d/%Y").is_ok() // MM/DD/YYYY
            },
            _ => true // Text type accepts any value
        }
    }
}

fn parse_fields(fields: Vec<Value>) -> Result<Vec<Field>, String> {
    fields.into_iter().map(|field| {
        let obj = field.as_object().ok_or("Field is not an object")?;
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Field name not found or not a string")?
            .to_string();
        let field_type = obj.get("type")
            .and_then(|v| v.as_str())
            .ok_or("Field type not found or not a string")?
            .to_string();
        
        Ok(Field { name, field_type })
    }).collect()
}

fn create_table_sql(table_name: &str, fields: &[Field]) -> String {
    let columns = fields.iter()
        .map(|field| format!("\"{}\" {}", field.name, field.to_postgres_type()))
        .collect::<Vec<_>>()
        .join(", ");
    
    format!("CREATE TABLE IF NOT EXISTS \"{}\" ({})", table_name, columns)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn check_postgres_table_exists(
    connection_string: String,
    table_name: String,
) -> Result<bool, String> {
    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Query to check if table exists
    let exists_query = format!(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)"
    );
    
    let exists: bool = client
        .query_one(&exists_query, &[&table_name])
        .await
        .map_err(|e| format!("Failed to check if table exists: {}", e))?
        .get(0);

    Ok(exists)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn import_csv_to_postgres(
    window: tauri::Window,
    connection_string: String,
    path_to_file: String,
    table_name: String,
    delimiter: String,
    linebreak: String,
    fields: Vec<Value>,
) -> Result<(), String> {
    println!("Starting import process");
    println!("File: {}", path_to_file);
    println!("Table: {}", table_name);
    println!("Delimiter: {}", delimiter);
    println!("Number of fields: {}", fields.len());

    // Parse fields into our internal representation
    let parsed_fields = parse_fields(fields)?;
    println!("Fields parsed successfully");
    
    // Generate CREATE TABLE SQL
    let create_table_sql = create_table_sql(&table_name, &parsed_fields);
    println!("Generated CREATE TABLE SQL: {}", create_table_sql);

    // Emit initial progress event
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: 0,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "connecting".to_string(),
            message: Some("Connecting to PostgreSQL".to_string()),
        },
    );

    // Connect to PostgreSQL
    println!("Connecting to PostgreSQL...");
    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| {
            println!("Connection error: {}", e);
            e.to_string()
        })?;

    println!("Connected successfully");

    // Spawn connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Create table if it doesn't exist
    println!("Creating table...");
    client
        .execute(&create_table_sql, &[])
        .await
        .map_err(|e| {
            println!("Failed to create table: {}", e);
            format!("Failed to create table: {}", e)
        })?;

    println!("Table created successfully");

    // Count total lines
    println!("Counting total rows...");
    let mut total_rows = 0;
    let file = std::fs::File::open(&path_to_file)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    
    // Count lines but skip the header
    for _ in reader.lines() {
        total_rows += 1;
    }
    total_rows -= 1; // Subtract header row
    
    println!("Total rows (excluding header): {}", total_rows);

    // Start COPY operation
    println!("Starting COPY operation...");
    let mut writer = start_copy(
        &client,
        &table_name,
        &parsed_fields.iter().map(|f| (f.name.clone(), f.field_type.clone())).collect::<Vec<_>>(),
        &delimiter
    )
    .await
    .map_err(|e| {
        println!("Failed to start COPY: {}", e);
        e.to_string()
    })?;

    // Write binary header
    let mut header_buf = BytesMut::new();
    write_binary_header(&mut header_buf);
    writer.send(header_buf).await.map_err(|e| {
        println!("Error writing binary header: {}", e);
        e.to_string()
    })?;

    let mut processed_rows = 0;
    const BATCH_SIZE: usize = 10000;

    // Process the file in streaming fashion
    println!("Processing file...");
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter.as_bytes()[0])
        .has_headers(true)
        .flexible(true)
        .from_path(&path_to_file)
        .map_err(|e| format!("Failed to create CSV reader: {}", e))?;

    // Get field types for binary conversion
    let field_types: Vec<String> = parsed_fields.iter().map(|f| f.field_type.clone()).collect();

    // Emit initial progress
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows: 0,
            row_count: 0,
            batch_size: BATCH_SIZE,
            status: "copying_data".to_string(),
            message: Some(format!("Starting import of {} rows...", total_rows)),
        },
    );

    let start_time = std::time::Instant::now();
    let mut last_progress_update = std::time::Instant::now();
    const PROGRESS_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

    let mut batch: Vec<BytesMut> = Vec::with_capacity(BATCH_SIZE);
    
    for result in reader.records() {
        if is_cancellation_requested() {
            println!("Cancellation requested");
            if let Err(e) = finish_copy(writer).await {
                eprintln!("Error finishing COPY operation during cancellation: {}", e);
            }
            return Err("Migration cancelled by user".to_string());
        }

        let record = match result {
            Ok(record) => record,
            Err(e) => {
                println!("Error reading record {}: {}", processed_rows + 1, e);
                continue;
            }
        };

        // Convert record to Vec<String>
        let record_values: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        
        // Create binary record
        let binary_record = prepare_binary_record(&record_values, &field_types);
        writer.send(binary_record).await.map_err(|e| {
            println!("Error writing record: {}", e);
            e.to_string()
        })?;

        processed_rows += 1;

        // Update progress periodically
        if last_progress_update.elapsed() >= PROGRESS_UPDATE_INTERVAL {
            let _ = window.emit(
                "migration_progress",
                ProgressEvent {
                    total_rows,
                    processed_rows,
                    row_count: processed_rows,
                    batch_size: BATCH_SIZE,
                    status: "copying_data".to_string(),
                    message: Some(format!(
                        "Imported {} of {} rows ({:.1}%)",
                        processed_rows,
                        total_rows,
                        (processed_rows as f64 / total_rows as f64) * 100.0
                    )),
                },
            );
            
            last_progress_update = std::time::Instant::now();
        }
    }

    // Write trailer
    let mut trailer_buf = BytesMut::new();
    trailer_buf.extend_from_slice(&(-1i16).to_be_bytes()); // End marker
    writer.send(trailer_buf).await.map_err(|e| {
        println!("Error writing trailer: {}", e);
        e.to_string()
    })?;

    // Finish COPY operation
    println!("Finishing COPY operation...");
    finish_copy(writer).await.map_err(|e| {
        println!("Error finishing COPY: {}", e);
        e.to_string()
    })?;

    println!("Import completed successfully");
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: processed_rows,  // Use actual final count
            processed_rows,
            row_count: processed_rows,
            batch_size: BATCH_SIZE,
            status: "complete".to_string(),
            message: Some(format!("Successfully imported {} rows", processed_rows)),
        },
    );

    Ok(())
}

// Helper function to write binary header
fn write_binary_header(header_buf: &mut BytesMut) {
    // Write header format (0)
    header_buf.extend_from_slice(&(0 as u8).to_be_bytes());
    // Write header flags (0)
    header_buf.extend_from_slice(&(0 as u16).to_be_bytes());
}

// Helper function to prepare binary record
fn prepare_binary_record(record_values: &[String], field_types: &[String]) -> BytesMut {
    let mut binary_record = BytesMut::new();
    
    // Write number of fields
    binary_record.extend_from_slice(&(record_values.len() as i16).to_be_bytes());
    
    for (value, field_type) in record_values.iter().zip(field_types.iter()) {
        match field_type.as_str() {
            "integer" => {
                let value: i32 = value.parse().unwrap_or(0);
                binary_record.extend_from_slice(&value.to_be_bytes());
            },
            "number" => {
                let value: f64 = value.parse().unwrap_or(0.0);
                binary_record.extend_from_slice(&value.to_be_bytes());
            },
            "date" => {
                if let Ok(value) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    // Convert date to PostgreSQL date format (days since 2000-01-01)
                    let epoch = chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
                    let days = value.signed_duration_since(epoch).num_days() as i32;
                    binary_record.extend_from_slice(&days.to_be_bytes());
                } else {
                    // Write NULL for invalid dates
                    binary_record.extend_from_slice(&(-1i32).to_be_bytes());
                }
            },
            _ => {
                let value_bytes = value.as_bytes();
                binary_record.extend_from_slice(&(value_bytes.len() as i32).to_be_bytes());
                binary_record.extend_from_slice(value_bytes);
            }
        }
    }
    binary_record
}
