// src/commands.rs

use crate::csv_reader; // new module for CSV reading
use crate::csv_schema; // your existing csv_schema module
use crate::sqlite_writer; // new module for SQLite writing
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::Emitter;

// This static remains here so we can set/cancel across the entire operation.
static CANCELLATION_REQUESTED: OnceLock<AtomicBool> = OnceLock::new();

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
pub async fn get_csv_schema(window: tauri::Window, file_path: String) -> Result<String, String> {
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
    let result = csv_schema::get_csv_schema(&file_path);

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
pub async fn csv_to_sqlite(
    window: tauri::Window,
    file_path: String,
    batch_size: usize,
    schema: String,
    db_path: String,
    table_name: String,
) -> Result<(), String> {
    // 1. Validate input parameters
    if !std::path::Path::new(&file_path).exists() {
        return Err(format!("File does not exist: {}", file_path));
    }
    if batch_size == 0 {
        return Err("Batch size must be greater than 0".to_string());
    }
    if schema.is_empty() {
        return Err("Schema cannot be empty".to_string());
    }
    if table_name.is_empty() {
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
    let connection = sqlite_writer::open_connection(&db_path)?;

    // 4. Create or ensure the table exists
    sqlite_writer::create_table(&connection, &table_name, &columns)?;

    // 5. Prepare the INSERT statement
    let mut statement = sqlite_writer::prepare_insert(&connection, &table_name, &columns)?;

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

    let total_rows = csv_reader::count_rows(&file_path)?;
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
    let delimiter = csv_reader::detect_delimiter(&file_path)?;
    let mut rdr = csv_reader::create_csv_reader(&file_path, delimiter)?;

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
                sqlite_writer::rollback_transaction(&connection); // Cleanup
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
                sqlite_writer::rollback_transaction(&connection); // Cleanup
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
                    message: None,
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
