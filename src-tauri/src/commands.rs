// src/commands.rs

use crate::csv_reader; // new module for CSV reading
use crate::csv_schema; // your existing csv_schema module
use crate::postgres_writer; // new module for PostgreSQL writing
use bytes::BytesMut;
use crate::sqlite_writer; // new module for SQLite writing
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::OnceLock;
use tauri::Emitter;

// This static remains here so we can set/cancel across the entire operation.
static CANCELLATION_REQUESTED: OnceLock<AtomicBool> = OnceLock::new();

pub fn is_cancellation_requested() -> bool {
    CANCELLATION_REQUESTED
        .get_or_init(|| AtomicBool::new(false))
        .load(Ordering::SeqCst)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    pub total_rows: usize,
    pub processed_rows: usize,
    pub row_count: usize,
    pub batch_size: usize,
    pub status: String,
    pub message: Option<String>,
}

#[tauri::command]
pub async fn cancel_migration() -> Result<(), String> {
    let flag = CANCELLATION_REQUESTED.get_or_init(|| AtomicBool::new(false));
    flag.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn reset_cancellation() -> Result<(), String> {
    let flag = CANCELLATION_REQUESTED.get_or_init(|| AtomicBool::new(false));
    flag.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn get_csv_schema(window: tauri::Window, filePath: String) -> Result<String, String> {
    // Emit event before schema parsing
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: 0,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "parsing_schema_start".to_string(),
            message: Some("Starting schema parsing".to_string()),
        },
    );

    // Delegate to csv_schema module
    let result = csv_schema::get_csv_schema(&filePath);

    // Emit event after schema parsing
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: 0,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "parsing_schema_complete".to_string(),
            message: Some("Schema parsing complete".to_string()),
        },
    );

    result
}

#[tauri::command]
pub async fn csv_to_postgres(
    window: tauri::Window,
    filePath: String,
    batch_size: usize,
    schema: String,
    dbPath: String,
    tableName: String,
) -> Result<(), String> {
    // 1. Validate input parameters
    if !std::path::Path::new(&filePath).exists() {
        return Err(format!("File does not exist: {}", filePath));
    }
    if batch_size == 0 {
        return Err("Batch size must be greater than 0".to_string());
    }
    if schema.is_empty() {
        return Err("Schema cannot be empty".to_string());
    }
    if tableName.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }

    // 2. Parse the schema string into column definitions
    let columns: Vec<(String, String)> = schema
        .split(',')
        .map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid schema format: expected 'name:type', got '{}'",
                    s
                ));
            }
            let name = parts[0].trim().to_string();
            let typ = parts[1].trim().to_string();
            if name.is_empty() || typ.is_empty() {
                return Err(format!("Empty name or type in schema: '{}'", s));
            }
            Ok((name, typ))
        })
        .collect::<Result<Vec<_>, String>>()?;

    // 3. Open PostgreSQL connection
    let client = Arc::new(postgres_writer::open_connection(&dbPath).await?);

    // 4. Create or ensure the table exists
    postgres_writer::create_table(&client, &tableName, &columns).await?;

    // 5. Begin COPY operation
    let delimiter = csv_reader::detect_delimiter(&filePath)?;
    let delimiter_str = if delimiter == b'\t' { "\t" } else { "," };
    let mut copy_writer = postgres_writer::start_copy(&client, &tableName, &columns, delimiter_str).await?;

    // 6. Count total rows (for progress reporting)
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: 0,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "counting_rows".to_string(),
            message: None,
        },
    );

    let total_rows = csv_reader::count_rows(&filePath)?;
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "counted_rows".to_string(),
            message: None,
        },
    );

    // 7. Detect delimiter and create a CSV reader
    let mut rdr = csv_reader::create_csv_reader(&filePath, delimiter)?;

    // Reset cancellation flag at the start of migration
    if let Some(flag) = CANCELLATION_REQUESTED.get() {
        flag.store(false, Ordering::SeqCst);
    }

    // 8. Process CSV rows using COPY
    let mut processed_rows = 0;
    let mut row_count = 0;
    let mut last_logged = 0;

    for result in rdr.records() {
        // Check for user cancellation
        if let Some(flag) = CANCELLATION_REQUESTED.get() {
            if flag.load(Ordering::SeqCst) {
                let _ = window.emit(
                    "migration_progress",
                    ProgressEvent {
                        total_rows,
                        processed_rows,
                        row_count,
                        batch_size: 0,
                        status: "cancelled".to_string(),
                        message: Some("Migration cancelled by user".to_string()),
                    },
                );
                // Reset the cancellation flag
                flag.store(false, Ordering::SeqCst);
                return Ok(());
            }
        }

        row_count += 1;
        processed_rows += 1;

        // Log progress every 100,000 rows
        if row_count - last_logged >= 100_000 {
            last_logged = row_count;
            let _ = window.emit(
                "migration_progress",
                ProgressEvent {
                    total_rows,
                    processed_rows,
                    row_count,
                    batch_size,
                    status: "processing".to_string(),
                    message: Some(format!(
                        "Processed {} rows ({:.1}%)",
                        processed_rows,
                        (processed_rows as f64 / total_rows as f64) * 100.0
                    )),
                },
            );
        }

        let record = match result {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Error processing row {}: {}", row_count, e));
            }
        };

        // Convert the record to a tab-separated string
        let copy_line = record.iter()
            .map(|field| {
                if field.is_empty() {
                    "\\N".to_string() // PostgreSQL NULL value
                } else {
                    // Quote and escape the field
                    format!("\"{}\"", field.replace('"', "\"\""))
                }
            })
            .collect::<Vec<_>>()
            .join("\t");

        // Write the record using the COPY protocol
        postgres_writer::write_copy_row(&mut copy_writer, BytesMut::from(copy_line.as_bytes())).await?;
    }

    // Finish COPY operation
    postgres_writer::finish_copy(copy_writer).await?;

    // Final progress event
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows,
            row_count: processed_rows,
            batch_size: 0,
            status: "complete".to_string(),
            message: Some(format!("Successfully copied {} rows", processed_rows)),
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn csv_to_sqlite(
    window: tauri::Window,
    filePath: String,
    batch_size: usize,
    schema: String,
    dbPath: String,
    tableName: String,
) -> Result<(), String> {
    // 1. Validate input parameters
    if !std::path::Path::new(&filePath).exists() {
        return Err(format!("File does not exist: {}", filePath));
    }
    if batch_size == 0 {
        return Err("Batch size must be greater than 0".to_string());
    }
    if schema.is_empty() {
        return Err("Schema cannot be empty".to_string());
    }
    if tableName.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }

    // 2. Parse the schema string into column definitions
    let columns: Vec<(String, String)> = schema
        .split(',')
        .map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid schema format: expected 'name:type', got '{}'",
                    s
                ));
            }
            let name = parts[0].trim().to_string();
            let typ = parts[1].trim().to_string();
            if name.is_empty() || typ.is_empty() {
                return Err(format!("Empty name or type in schema: '{}'", s));
            }
            Ok((name, typ))
        })
        .collect::<Result<Vec<_>, String>>()?;

    // 3. Open and configure the SQLite database
    let connection = sqlite_writer::open_connection(&dbPath)?;

    // 4. Create or ensure the table exists
    sqlite_writer::create_table(&connection, &tableName, &columns)?;

    // 5. Prepare the INSERT statement
    let mut statement = sqlite_writer::prepare_insert(&connection, &tableName, &columns)?;

    // 6. Begin initial transaction
    sqlite_writer::begin_transaction(&connection)?;

    // 7. Count total rows (for progress reporting)
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: 0,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "counting_rows".to_string(),
            message: None,
        },
    );

    let total_rows = csv_reader::count_rows(&filePath)?;
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "counted_rows".to_string(),
            message: None,
        },
    );

    // 8. Detect delimiter and create a CSV reader
    let delimiter = csv_reader::detect_delimiter(&filePath)?;
    let mut rdr = csv_reader::create_csv_reader(&filePath, delimiter)?;

    // Reset cancellation flag at the start of migration
    if let Some(flag) = CANCELLATION_REQUESTED.get() {
        flag.store(false, Ordering::SeqCst);
    }

    // 9. Process CSV rows, inserting in batches
    let mut processed_rows = 0;
    let mut row_count = 0;
    let mut last_logged = 0;

    for result in rdr.records() {
        // Check for user cancellation
        if let Some(flag) = CANCELLATION_REQUESTED.get() {
            if flag.load(Ordering::SeqCst) {
                let _ = sqlite_writer::rollback_transaction(&connection); // Cleanup
                let _ = window.emit(
                    "migration_progress",
                    ProgressEvent {
                        total_rows,
                        processed_rows,
                        row_count,
                        batch_size: 0,
                        status: "cancelled".to_string(),
                        message: Some("Migration cancelled by user".to_string()),
                    },
                );
                // Reset the cancellation flag
                flag.store(false, Ordering::SeqCst);
                return Ok(());
            }
        }

        row_count += 1;
        processed_rows += 1;

        // Log progress every 100,000 rows
        if row_count - last_logged >= 100_000 {
            // or just log to console if you wish
            last_logged = row_count;
        }

        let record = match result {
            Ok(r) => r,
            Err(e) => {
                let _ = sqlite_writer::rollback_transaction(&connection); // Cleanup
                return Err(format!("Error processing row {}: {}", row_count, e));
            }
        };

        // Insert the current record into SQLite
        sqlite_writer::insert_record(&mut statement, &record)?;

        // Commit the batch if we've reached the batch_size
        if processed_rows % batch_size == 0 {
            sqlite_writer::commit_and_begin_new_transaction(&connection)?;
            let _ = window.emit(
                "migration_progress",
                ProgressEvent {
                    total_rows,
                    processed_rows,
                    row_count,
                    batch_size,
                    status: "processing".to_string(),
                    message: Some(format!(
                        "Processed {} rows ({:.1}%)",
                        processed_rows,
                        (processed_rows as f64 / total_rows as f64) * 100.0
                    )),
                },
            );
        }
    }

    // Final commit
    sqlite_writer::commit_transaction(&connection)?;

    // Final progress event
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows,
            row_count,
            batch_size: 0,
            status: "complete".to_string(),
            message: None,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn read_file_chunks(filePath: String, chunkSize: usize, offset: usize) -> Result<(Vec<String>, bool), String> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::time::Instant;

    let start = Instant::now();
    println!("Starting to read file chunks from offset {}", offset);
    
    let file = File::open(&filePath).await.map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    // Skip to offset
    for _ in 0..offset {
        if lines.next_line().await.map_err(|e| e.to_string())?.is_none() {
            return Ok((Vec::new(), true));
        }
    }
    
    let mut chunks = Vec::new();
    let mut current_chunk = String::with_capacity(chunkSize * 100);
    let mut line_count = 0;
    let mut total_lines = 0;
    let batch_size = 10;
    let target_lines = chunkSize * batch_size;
    
    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        current_chunk.push_str(&line);
        current_chunk.push('\n');
        line_count += 1;
        total_lines += 1;

        if line_count >= chunkSize {
            chunks.push(current_chunk);
            current_chunk = String::with_capacity(chunkSize * 100);
            line_count = 0;
        }

        if total_lines >= target_lines {
            break;
        }
    }

    // Push the last chunk if it's not empty
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    // Check if there are more lines
    let is_last_batch = lines.next_line().await.map_err(|e| e.to_string())?.is_none();
    
    println!("Read {} chunks in {:?}", chunks.len(), start.elapsed());
    Ok((chunks, is_last_batch))
}
