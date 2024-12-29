// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod csv_schema;
mod db;

use db::connect_db;
use rust_decimal::Decimal;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path;
use time::OffsetDateTime;
use tokio_postgres::types::Type;
use tokio_postgres::{NoTls, Row};
use uuid::Uuid;

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
async fn connect(url: String) -> Result<(), String> {
    match connect_db(&url).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            connect,
            db::test_database_connection,
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
    file_path: String,
    batch_size: usize,
    schema: String,
    db_path: String,
) -> Result<(), String> {
    // Parse schema string into column names and types
    let columns: Vec<(String, String)> = schema
        .split(',')
        .map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid schema format: {}", s));
            }
            Ok((parts[0].to_string(), parts[1].to_string()))
        })
        .collect::<Result<Vec<_>, String>>()?;

    // Create table if it doesn't exist
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
    execute_sqlite_query(db_path.clone(), create_table_sql).await?;

    // Open CSV file
    let file = std::fs::File::open(&file_path).map_err(|e| e.to_string())?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut batch = Vec::new();
    let mut total_rows = 0;
    let mut row_count = 0;

    println!("Starting CSV processing...");
    for result in rdr.records() {
        row_count += 1;
        // println!("Processing row {}", row_count);

        let record = match result {
            Ok(r) => r,
            Err(e) => {
                println!("Error reading CSV row {}: {}", row_count, e);
                return Err(e.to_string());
            }
        };

        //// println!("Row {}: {:?}", row_count, record);

        // Convert record to SQL values
        let values: Vec<String> = record
            .iter()
            .map(|field| {
                if field.is_empty() {
                    "NULL".to_string()
                } else {
                    format!("'{}'", field.replace("'", "''"))
                }
            })
            .collect();

        //// println!("Row {} values: {:?}", row_count, values);

        batch.push(format!("({})", values.join(", ")));
        ////println!("Current batch size: {}", batch.len());

        // Execute batch when size is reached
        if batch.len() >= batch_size {
            let insert_sql = format!(
                "INSERT INTO {} ({}) VALUES {}",
                table_name,
                columns
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                batch.join(", ")
            );
            println!("Executing batch insert of {} rows", batch.len());
            match execute_sqlite_query(db_path.clone(), insert_sql).await {
                Ok(result) => {
                    println!("Successfully inserted {} rows", result.rows.len());
                    total_rows += batch.len();
                    batch.clear();
                }
                Err(e) => {
                    println!("Error inserting batch: {}", e);
                    return Err(e);
                }
            }
        }
    }

    // Insert remaining records
    if !batch.is_empty() {
        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table_name,
            columns
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            batch.join(", ")
        );
        println!("Executing final batch insert of {} rows", batch.len());
        match execute_sqlite_query(db_path.clone(), insert_sql).await {
            Ok(result) => {
                println!("Successfully inserted {} rows", result.rows.len());
                total_rows += batch.len();
            }
            Err(e) => {
                println!("Error inserting final batch: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_csv_schema(file_path: String) -> Result<String, String> {
    csv_schema::get_csv_schema(&file_path)
}
