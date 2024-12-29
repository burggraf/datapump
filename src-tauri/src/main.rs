// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod csv_schema;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::path;
use tauri::Emitter;
use time::OffsetDateTime;
use tokio_postgres::types::Type;
use tokio_postgres::{NoTls, Row};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProgressEvent {
    total_rows: usize,
    processed_rows: usize,
    row_count: usize,
    batch_size: usize,
    status: String,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

async fn format_row_value(row: &Row, i: usize, col_type: &Type) -> String {
    match col_type {
        &Type::BOOL => match row.get::<_, Option<bool>>(i) {
            Some(b) => b.to_string(),
            None => "NULL".to_string(),
        },
        &Type::INT2 => match row.get::<_, Option<i16>>(i) {
            Some(n) => n.to_string(),
            None => "NULL".to_string(),
        },
        &Type::INT4 => match row.get::<_, Option<i32>>(i) {
            Some(n) => n.to_string(),
            None => "NULL".to_string(),
        },
        &Type::INT8 => match row.get::<_, Option<i64>>(i) {
            Some(n) => n.to_string(),
            None => "NULL".to_string(),
        },
        &Type::FLOAT4 | &Type::FLOAT8 => match row.get::<_, Option<f64>>(i) {
            Some(f) => f.to_string(),
            None => "NULL".to_string(),
        },
        &Type::NUMERIC => match row.get::<_, Option<Decimal>>(i) {
            Some(d) => d.to_string(),
            None => "NULL".to_string(),
        },
        &Type::UUID => match row.get::<_, Option<Uuid>>(i) {
            Some(uuid) => uuid.to_string(),
            None => "NULL".to_string(),
        },
        &Type::TIMESTAMP | &Type::TIMESTAMPTZ => match row.get::<_, Option<OffsetDateTime>>(i) {
            Some(ts) => ts.to_string(),
            None => "NULL".to_string(),
        },
        &Type::JSON | &Type::JSONB => match row.get::<_, Option<serde_json::Value>>(i) {
            Some(json) => json.to_string(),
            None => "NULL".to_string(),
        },
        _ => match row.get::<_, Option<String>>(i) {
            Some(s) => s,
            None => "NULL".to_string(),
        },
    }
}

#[tauri::command]
async fn execute_postgres_query(
    connection_string: String,
    query: String,
) -> Result<QueryResult, String> {
    println!("Connecting to database...");
    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| {
            println!("Connection error: {}", e);
            e.to_string()
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Executing query: {}", query);
    let rows = match client.query(query.as_str(), &[]).await {
        Ok(r) => {
            println!("Query successful, got {} rows", r.len());
            r
        }
        Err(e) => {
            println!("Query error: {}", e);
            return Err(e.to_string());
        }
    };

    if rows.is_empty() {
        println!("No rows returned");
        return Ok(QueryResult {
            columns: vec![],
            rows: vec![],
        });
    }

    println!("Processing {} rows", rows.len());
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|col| col.name().to_string())
        .collect();

    let mut formatted_rows = Vec::new();
    for row in &rows {
        let mut formatted_row = Vec::new();
        for (i, column) in row.columns().iter().enumerate() {
            let value = format_row_value(row, i, column.type_()).await;
            formatted_row.push(value);
        }
        formatted_rows.push(formatted_row);
    }

    println!("Query complete, returning {} rows", formatted_rows.len());
    Ok(QueryResult {
        columns,
        rows: formatted_rows,
    })
}

#[tauri::command]
async fn execute_sqlite_query(
    connection_string: String,
    query: String,
) -> Result<QueryResult, String> {
    // println!("Connecting to sqlite database...");
    let connection = sqlite::open(connection_string).map_err(|e| e.to_string())?;

    // println!("Executing query: {}", query);
    let mut statement = connection.prepare(query).map_err(|e| e.to_string())?;

    let mut columns: Vec<String> = Vec::new();
    for i in 0..statement.column_count() {
        columns.push(statement.column_name(i).unwrap().to_string());
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    while let Ok(sqlite::State::Row) = statement.next() {
        let mut row: Vec<String> = Vec::new();
        for i in 0..statement.column_count() {
            let value = statement
                .read::<String, usize>(i)
                .unwrap_or("NULL".to_string());
            row.push(value);
        }
        rows.push(row);
    }

    // println!("Query complete, returning {} rows", rows.len());
    Ok(QueryResult { columns, rows })
}

#[tauri::command]
async fn get_real_path(file_path: String) -> Result<String, String> {
    let absolute_path = path::absolute(file_path).map_err(|e| e.to_string())?;
    Ok(absolute_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn append_to_file(file_path: String, text: String) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| e.to_string())?;

    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_csv_schema(window: tauri::Window, file_path: String) -> Result<String, String> {
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            execute_postgres_query,
            execute_sqlite_query,
            get_real_path,
            append_to_file,
            get_csv_schema,
            csv_to_sqlite
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn csv_to_sqlite(
    window: tauri::Window,
    file_path: String,
    batch_size: usize,
    schema: String,
    db_path: String,
) -> Result<(), String> {
    println!("Validating parameters...");
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
    println!("Parameters validated successfully");

    println!("Parsing schema...");
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
    println!("Schema parsed successfully");

    println!("Opening database connection...");
    let connection = sqlite::open(&db_path).map_err(|e| {
        println!("Failed to open database: {}", e);
        e.to_string()
    })?;
    println!("Database connection established");

    println!("Setting WAL mode...");
    connection
        .execute("PRAGMA journal_mode=WAL;")
        .map_err(|e| {
            println!("Failed to set WAL mode: {}", e);
            e.to_string()
        })?;
    println!("WAL mode set successfully");

    println!("Setting synchronous mode...");
    connection
        .execute("PRAGMA synchronous=NORMAL;")
        .map_err(|e| {
            println!("Failed to set synchronous mode: {}", e);
            e.to_string()
        })?;
    println!("Synchronous mode set successfully");

    println!("Creating table...");
    let table_name = "imported_data";
    let create_table_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table_name,
        columns
            .iter()
            .map(|(name, typ)| format!("{} {}", name, typ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("Executing SQL: {}", create_table_sql);
    connection.execute(create_table_sql).map_err(|e| {
        println!("Failed to create table: {}", e);
        e.to_string()
    })?;
    println!("Table created successfully");

    println!("Preparing insert statement...");
    let placeholders = columns.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let column_names = columns
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>()
        .join(",");
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name, column_names, placeholders
    );
    println!("Insert SQL: {}", insert_sql);
    let mut statement = connection.prepare(&insert_sql).map_err(|e| {
        println!("Failed to prepare insert statement: {}", e);
        e.to_string()
    })?;
    println!("Insert statement prepared successfully");

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

    println!("Starting transaction...");
    execute_with_retry(&connection, |conn| {
        conn.execute("BEGIN IMMEDIATE TRANSACTION").map_err(|e| {
            println!("Failed to start transaction: {}", e);
            e.to_string()
        })
    })?;
    println!("Transaction started successfully");

    println!("Counting total rows...");
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
    println!("Total rows to process: {}", total_rows);

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

    println!("Resetting CSV reader...");
    let file = std::fs::File::open(&file_path).map_err(|e| {
        println!("Failed to reopen CSV file: {}", e);
        e.to_string()
    })?;
    let mut rdr = csv::Reader::from_reader(file);
    println!("CSV reader reset successfully");

    println!("Starting CSV processing...");
    let mut last_logged = 0;
    for result in rdr.records() {
        row_count += 1;
        processed_rows += 1;

        // Log progress every 100,000 rows
        if row_count - last_logged >= 100000 {
            println!("Processed {} rows...", row_count);
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
