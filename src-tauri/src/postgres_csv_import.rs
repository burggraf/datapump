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
    // Debug print the new parameters
    println!("Received delimiter: {}", delimiter);
    println!("Received linebreak: {}", linebreak);
    println!("Received fields: {:?}", fields);

    // Parse fields into our internal representation
    let parsed_fields = parse_fields(fields)?;
    println!("Parsed fields: {:?}", parsed_fields);
    
    // Generate CREATE TABLE SQL
    let create_table_sql = create_table_sql(&table_name, &parsed_fields);
    println!("Generated SQL: {}", create_table_sql);

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
    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| e.to_string())?;

    // Spawn connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Check if table exists
    let exists = check_postgres_table_exists(connection_string.clone(), table_name.clone()).await?;
    if exists {
        return Err(format!("Table '{}' already exists. Aborting import.", table_name));
    }

    // Create table if it doesn't exist
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows: 0,
            processed_rows: 0,
            row_count: 0,
            batch_size: 0,
            status: "creating_table".to_string(),
            message: Some("Creating table if not exists".to_string()),
        },
    );

    client
        .execute(&create_table_sql, &[])
        .await
        .map_err(|e| format!("Failed to create table: {}", e))?;

    println!("Table created successfully");

    // Read the entire file into memory
    let mut file = File::open(&path_to_file).await.map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.map_err(|e| e.to_string())?;

    // Convert Windows-1252 to UTF-8
    let content = {
        let mut result = String::with_capacity(buffer.len());
        for &byte in &buffer {
            let c = match byte {
                0xF3 => 'ó',  // Handle ó character
                0xF2 => 'ò',  // Handle ò character
                0xF1 => 'ñ',  // Handle ñ character
                0xE1 => 'á',  // Handle á character
                0xE9 => 'é',  // Handle é character
                0xED => 'í',  // Handle í character
                0xFA => 'ú',  // Handle ú character
                0x0D => '\r', // Handle CR
                0x0A => '\n', // Handle LF
                _ => char::from(byte),  // Convert u8 to char
            };
            result.push(c);
        }
        result
    };

    // Convert the linebreak string to actual characters
    let line_delimiter = match linebreak.as_str() {
        "\\r\\n" | "\r\n" => "\r\n",
        "\\r" | "\r" => "\r",
        "\\n" | "\n" => "\n",
        _ => "\n", // Default to \n if unspecified
    };
    
    println!("Using line delimiter: {:?}", line_delimiter);
    println!("Content length: {}", content.len());

    // Split into lines based on the specified line delimiter
    let lines: Vec<&str> = content.split(line_delimiter)
        .filter(|line| !line.is_empty())
        .collect();
    
    println!("Number of lines after splitting: {}", lines.len());
    if !lines.is_empty() {
        println!("First line length: {}", lines[0].len());
        println!("First line preview: {:?}", &lines[0][..std::cmp::min(100, lines[0].len())]);
    }

    let total_rows = lines.len().saturating_sub(1); // Subtract 1 for header
    println!("Total rows (excluding header): {}", total_rows);

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

    // Start COPY operation
    let mut writer = start_copy(
        &client,
        &table_name,
        &parsed_fields.iter().map(|f| (f.name.clone(), f.field_type.clone())).collect::<Vec<_>>(),
        &delimiter
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut processed_rows = 0;
    let mut is_header = true;

    // Process each line
    for line in lines {
        if is_cancellation_requested() {
            if let Err(e) = finish_copy(writer).await {
                eprintln!("Error finishing COPY operation during cancellation: {}", e);
            }
            return Err("Migration cancelled by user".to_string());
        }

        // Skip header row
        if is_header {
            is_header = false;
            continue;
        }

        // Parse the line using csv crate
        let mut csv_reader = ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .flexible(true)
            .trim(Trim::All)
            .from_reader(line.as_bytes());

        println!("Parsing line: {:?}", line);
        
        let mut valid_line = true;
        let mut validated_fields = Vec::new();

        if let Some(result) = csv_reader.records().next() {
            match result {
                Ok(record) => {
                    println!("Successfully parsed record with {} fields", record.len());
                    for (i, field) in record.iter().enumerate() {
                        if i >= parsed_fields.len() {
                            valid_line = false;
                            eprintln!("Row {}: Too many fields (got {}, expected {})", 
                                processed_rows + 1, record.len(), parsed_fields.len());
                            break;
                        }

                        println!("Field {}: {:?} (type: {})", 
                            i, field, parsed_fields[i].field_type);

                        if !parsed_fields[i].validate_value(field) {
                            valid_line = false;
                            eprintln!(
                                "Row {}: Invalid value '{}' for {} field '{}'",
                                processed_rows + 1,
                                field,
                                parsed_fields[i].field_type,
                                parsed_fields[i].name
                            );
                            break;
                        }
                        validated_fields.push(field.to_string());
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing row {}: {}", processed_rows + 1, e);
                    valid_line = false;
                }
            }
        }

        if !valid_line {
            processed_rows += 1;
            continue;
        }

        // Construct a valid PostgreSQL COPY line
        let copy_line = validated_fields
            .iter()
            .map(|f| {
                if f.is_empty() {
                    "\\N".to_string() // PostgreSQL NULL value
                } else if parsed_fields[validated_fields.iter().position(|x| x == f).unwrap()].field_type == "integer" {
                    // For integer fields, don't quote them
                    f.to_string()
                } else {
                    // For text fields, quote and escape
                    format!("\"{}\"", f.replace('"', "\"\""))
                }
            })
            .collect::<Vec<_>>()
            .join("\t");  // Use tab as the COPY delimiter

        // Add the newline
        let mut line_bytes = BytesMut::from(copy_line.as_bytes());
        line_bytes.extend_from_slice(b"\n");
        writer.send(line_bytes).await.map_err(|e| e.to_string())?;
        
        processed_rows += 1;

        // Emit progress every 1000 rows
        if processed_rows % 1000 == 0 {
            let _ = window.emit(
                "migration_progress",
                ProgressEvent {
                    total_rows,
                    processed_rows,
                    row_count: processed_rows,
                    batch_size: 1000,
                    status: "processing".to_string(),
                    message: Some(format!("Processed {} rows", processed_rows)),
                },
            );
        }
    }

    // Finish COPY operation
    finish_copy(writer).await.map_err(|e| e.to_string())?;

    // Final progress event
    let _ = window.emit(
        "migration_progress",
        ProgressEvent {
            total_rows,
            processed_rows,
            row_count: processed_rows,
            batch_size: 0,
            status: "complete".to_string(),
            message: None,
        },
    );

    Ok(())
}
