use futures_util::SinkExt;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncBufReadExt, BufReader, AsyncSeekExt};
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
use std::io::{BufRead, SeekFrom};

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

    // Count total lines using sync I/O
    println!("Counting total rows...");
    let mut total_rows = 0;
    {
        let file = std::fs::File::open(&path_to_file)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = std::io::BufReader::new(file);
        
        // Count lines but skip the header
        for _ in reader.lines() {
            total_rows += 1;
        }
        total_rows -= 1; // Subtract header row
    }
    println!("Total rows (excluding header): {}", total_rows);

    // Now open file asynchronously for processing
    let file = tokio::fs::File::open(&path_to_file)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;

    // Create buffered reader with custom buffer size
    const BUFFER_SIZE: usize = 8 * 1024 * 1024;
    let reader = tokio::io::BufReader::with_capacity(BUFFER_SIZE, file);
    let mut lines = reader.lines();

    // Skip header line
    if let Some(header) = lines.next_line().await.map_err(|e| e.to_string())? {
        println!("Skipped header: {}", header);
    }

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
    let mut last_progress_update = std::time::Instant::now();
    const PROGRESS_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

    // Pre-allocate reusable buffers with larger sizes
    let mut binary_buffer = BytesMut::with_capacity(256 * 1024);  // 256KB for binary buffer
    let mut batch_buffer = BytesMut::with_capacity(2 * 1024 * 1024);  // 2MB for batching

    // Pre-compute field processors
    let field_processors: Vec<FieldProcessor> = parsed_fields
        .iter()
        .enumerate()
        .map(|(i, ft)| FieldProcessor {
            field_type: ft.field_type.clone(),
            index: i,
        })
        .collect();

    let mut batch_count = 0;
    const BATCH_THRESHOLD: usize = 5000; // Increased batch size
    
    let delim = delimiter.as_bytes()[0];

    while let Some(line_result) = lines.next_line().await.map_err(|e| e.to_string())? {
        if is_cancellation_requested() {
            println!("Cancellation requested");
            if let Err(e) = finish_copy(writer).await {
                eprintln!("Error finishing COPY operation during cancellation: {}", e);
            }
            return Err("Migration cancelled by user".to_string());
        }

        // Clear buffers for reuse
        binary_buffer.clear();

        // Process the line directly without storing references
        let bytes = line_result.as_bytes();
        let mut start = 0;
        let mut field_count = 0;

        // Write placeholder for field count
        binary_buffer.extend_from_slice(&0i16.to_be_bytes());
        
        // Process each field directly
        for i in 0..bytes.len() {
            if bytes[i] == delim {
                if let Some(processor) = field_processors.get(field_count) {
                    let field = unsafe { std::str::from_utf8_unchecked(&bytes[start..i]) };
                    processor.process_value(field, &mut binary_buffer);
                }
                start = i + 1;
                field_count += 1;
            }
        }
        
        // Process the last field
        if let Some(processor) = field_processors.get(field_count) {
            let field = unsafe { std::str::from_utf8_unchecked(&bytes[start..]) };
            processor.process_value(field, &mut binary_buffer);
            field_count += 1;
        }

        // Write actual field count at the start
        let field_count_bytes = (field_count as i16).to_be_bytes();
        binary_buffer[0] = field_count_bytes[0];
        binary_buffer[1] = field_count_bytes[1];

        // Add to batch buffer
        batch_buffer.extend_from_slice(&binary_buffer);
        batch_count += 1;

        // Send batch if threshold reached
        if batch_count >= BATCH_THRESHOLD {
            writer.send(batch_buffer.split_to(batch_buffer.len())).await.map_err(|e| {
                println!("Error writing batch: {}", e);
                e.to_string()
            })?;
            batch_count = 0;
        }

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
            total_rows,
            processed_rows,
            row_count: processed_rows,
            batch_size: BATCH_SIZE,
            status: "complete".to_string(),
            message: Some(format!("Successfully imported {} rows", processed_rows)),
        },
    );

    Ok(())
}

struct FieldProcessor {
    field_type: String,
    index: usize,
}

impl FieldProcessor {
    fn process_value(&self, value: &str, binary_record: &mut BytesMut) {
        match self.field_type.as_str() {
            "integer" => {
                if let Ok(value) = value.parse::<i32>() {
                    binary_record.extend_from_slice(&value.to_be_bytes());
                } else {
                    binary_record.extend_from_slice(&0i32.to_be_bytes());
                }
            },
            "number" => {
                if let Ok(value) = value.parse::<f64>() {
                    binary_record.extend_from_slice(&value.to_be_bytes());
                } else {
                    binary_record.extend_from_slice(&0f64.to_be_bytes());
                }
            },
            "date" => {
                if let Ok(value) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    let epoch = chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
                    let days = value.signed_duration_since(epoch).num_days() as i32;
                    binary_record.extend_from_slice(&days.to_be_bytes());
                } else {
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
}

// Helper function to write binary header
fn write_binary_header(header_buf: &mut BytesMut) {
    // Write header format (0)
    header_buf.extend_from_slice(&(0 as u8).to_be_bytes());
    // Write header flags (0)
    header_buf.extend_from_slice(&(0 as u16).to_be_bytes());
}
