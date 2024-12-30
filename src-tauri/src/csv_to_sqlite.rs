use super::csv_schema;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::Emitter;

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
    // println!("Validating parameters...");
    if !std::path::Path::new(&file_path).exists() {
        println!("File does not exist: {}", file_path);
        return Err(format!("File does not exist: {}", file_path));
    }
    if batch_size == 0 {
        println!("Invalid batch size: 0");
        return Err("Batch size must be greater than 0".to_string());
    }
    if schema.is_empty() {
        println!("Empty schema provided");
        return Err("Schema cannot be empty".to_string());
    }

    // Validate table name
    if table_name.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }
    if !table_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(
            "Table name must contain only alphanumeric characters and underscores".to_string(),
        );
    }
    if table_name.chars().next().unwrap().is_ascii_digit() {
        return Err("Table name cannot start with a digit".to_string());
    }

    // println!("Parameters validated successfully");

    // println!("Parsing schema...");
    let columns: Vec<(String, String)> = schema
        .split(',')
        .map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 2 {
                println!("Invalid schema format: expected 'name:type', got '{}'", s);
                return Err(format!(
                    "Invalid schema format: expected 'name:type', got '{}'",
                    s
                ));
            }
            let name = parts[0].trim().to_string();
            let typ = parts[1].trim().to_string();
            if name.is_empty() || typ.is_empty() {
                println!("Empty name or type in schema: '{}'", s);
                return Err(format!("Empty name or type in schema: '{}'", s));
            }
            Ok((name, typ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    // println!("Schema parsed successfully");

    // println!("Opening database connection...");
    let connection = sqlite::open(&db_path).map_err(|e| {
        println!("Failed to open database: {}", e);
        e.to_string()
    })?;
    // println!("Database connection established");

    // println!("Setting WAL mode...");
    connection
        .execute("PRAGMA journal_mode=WAL;")
        .map_err(|e| {
            println!("Failed to set WAL mode: {}", e);
            e.to_string()
        })?;
    // println!("WAL mode set successfully");

    // println!("Setting synchronous mode...");
    connection
        .execute("PRAGMA synchronous=NORMAL;")
        .map_err(|e| {
            println!("Failed to set synchronous mode: {}", e);
            e.to_string()
        })?;
    // println!("Synchronous mode set successfully");

    // !("Creating table...");
    let create_table_sql = format!(
        "CREATE TABLE IF NOT EXISTS \"{}\" ({})",
        table_name,
        columns
            .iter()
            .map(|(name, typ)| format!("\"{}\" {}", name, typ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    // println!("Executing SQL: {}", create_table_sql);
    connection.execute(create_table_sql).map_err(|e| {
        println!("Failed to create table: {}", e);
        e.to_string()
    })?;
    // println!("Table created successfully");

    // println!("Preparing insert statement...");
    let placeholders = columns.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let column_names = columns
        .iter()
        .map(|(name, _)| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(",");
    let insert_sql = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({})",
        table_name, column_names, placeholders
    );
    // println!("Insert SQL: {}", insert_sql);
    let mut statement = connection.prepare(&insert_sql).map_err(|e| {
        println!("Failed to prepare insert statement: {}", e);
        e.to_string()
    })?;
    // println!("Insert statement prepared successfully");

    // Function to execute with retry
    fn execute_with_retry<F>(connection: &sqlite::Connection, f: F) -> Result<(), String>
    where
        F: Fn(&sqlite::Connection) -> Result<(), String>,
    {
        let mut retries = 5;
        loop {
            match f(connection) {
                Ok(_) => return Ok(()),
                Err(e) if e.contains("database is locked") && retries > 0 => {
                    retries -= 1;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    // println!("Starting transaction...");
    execute_with_retry(&connection, |conn| {
        conn.execute("BEGIN IMMEDIATE TRANSACTION").map_err(|e| {
            println!("Failed to start transaction: {}", e);
            e.to_string()
        })
    })?;
    // println!("Transaction started successfully");

    // println!("Counting total rows...");
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

    let file = std::fs::File::open(&file_path).map_err(|e| {
        println!("Failed to open CSV file for row count: {}", e);
        e.to_string()
    })?;
    let reader = std::io::BufReader::new(file);

    let total_rows = reader.lines().count();
    // println!("Total rows to process: {}", total_rows);

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
    let mut processed_rows = 0;
    let mut row_count = 0;
    let batch_size = batch_size; // Use the user-provided batch size

    // Detect delimiter by analyzing first line
    let file = std::fs::File::open(&file_path).map_err(|e| {
        println!("Failed to reopen CSV file: {}", e);
        e.to_string()
    })?;
    let mut reader = std::io::BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).map_err(|e| {
        println!("Failed to read first line: {}", e);
        e.to_string()
    })?;

    // Count commas and tabs in first line
    let comma_count = first_line.matches(',').count();
    let tab_count = first_line.matches('\t').count();

    // Use tab delimiter if more tabs than commas, otherwise use comma
    let delimiter = if tab_count > comma_count { b'\t' } else { b',' };

    // Reset file reader
    let file = std::fs::File::open(&file_path).map_err(|e| {
        println!("Failed to reopen CSV file: {}", e);
        e.to_string()
    })?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .flexible(false)
        .from_reader(file);
    // println!("CSV reader reset successfully");

    // println!("Starting CSV processing...");
    let mut last_logged = 0;
    // Reset cancellation flag at start
    if let Some(flag) = CANCELLATION_REQUESTED.get() {
        flag.store(false, Ordering::SeqCst);
    }

    for result in rdr.records() {
        // Check for cancellation
        if let Some(flag) = CANCELLATION_REQUESTED.get() {
            if flag.load(Ordering::SeqCst) {
                connection.execute("ROLLBACK").ok();
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
                // Reset cancellation flag
                if let Some(flag) = CANCELLATION_REQUESTED.get() {
                    flag.store(false, Ordering::SeqCst);
                }
                return Ok(());
            }
        }
        row_count += 1;
        processed_rows += 1;

        // Log progress every 100,000 rows
        if row_count - last_logged >= 100000 {
            // println!("Processed {} rows...", row_count);
            last_logged = row_count;
        }

        let record = match result {
            Ok(r) => r,
            Err(e) => {
                println!("Error processing row {}: {}", row_count, e);
                connection.execute("ROLLBACK").ok();
                return Err(format!("Error processing row {}: {}", row_count, e));
            }
        };

        // Bind parameters
        for (i, field) in record.iter().enumerate() {
            if field.is_empty() {
                statement.bind((i + 1, ())).map_err(|e| e.to_string())?;
            } else {
                statement.bind((i + 1, field)).map_err(|e| e.to_string())?;
            }
        }

        // Execute insert
        statement.next().map_err(|e| e.to_string())?;
        // Reset statement for next row
        statement.reset().map_err(|e| e.to_string())?;

        // Commit batch
        if processed_rows % batch_size == 0 {
            match execute_with_retry(&connection, |conn| {
                conn.execute("COMMIT").map_err(|e| e.to_string())?;
                conn.execute("BEGIN IMMEDIATE TRANSACTION")
                    .map_err(|e| e.to_string())
            }) {
                Ok(_) => {
                    // Emit progress event
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
                Err(e) => {
                    println!("Error committing batch: {}", e);
                    return Err(e);
                }
            }
        }
    }

    // Final commit
    connection.execute("COMMIT").map_err(|e| e.to_string())?;

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
