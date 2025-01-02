// src/postgres_write.rs

use csv::StringRecord;
use bytes::BytesMut;
use futures_util::SinkExt;
use std::pin::Pin;
use tokio_postgres::{Client, CopyInSink, NoTls};

/// Open a PostgreSQL connection
pub async fn open_connection(connection_string: &str) -> Result<Client, String> {
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
        .await
        .map_err(|e| e.to_string())?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

/// Create table if not exists, given a table name and columns
pub async fn create_table(
    client: &Client,
    table_name: &str,
    columns: &[(String, String)],
) -> Result<(), String> {
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

    Ok(())
}

/// Start a COPY operation for bulk loading
pub async fn start_copy(
    client: &Client,
    table_name: &str,
    columns: &[(String, String)],
) -> Result<Pin<Box<CopyInSink<BytesMut>>>, String> {
    let column_names = columns
        .iter()
        .map(|(name, _)| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(", ");

    let copy_sql = format!("COPY {} ({}) FROM STDIN WITH CSV", table_name, column_names);
    
    let writer = client
        .copy_in(&copy_sql)
        .await
        .map_err(|e| format!("Failed to start COPY operation: {}", e))?;

    Ok(Box::pin(writer))
}

/// Write a CSV record using the COPY protocol
pub async fn copy_record(
    writer: &mut Pin<Box<CopyInSink<BytesMut>>>,
    record: &StringRecord,
) -> Result<(), String> {
    let mut line = String::new();
    for (i, field) in record.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        // Escape special characters and wrap in quotes if needed
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            line.push('"');
            for c in field.chars() {
                if c == '"' {
                    line.push('"'); // Double quotes to escape
                }
                line.push(c);
            }
            line.push('"');
        } else {
            line.push_str(field);
        }
    }
    line.push('\n');

    writer
        .send(BytesMut::from(line.as_bytes()))
        .await
        .map_err(|e| format!("Failed to write record: {}", e))?;

    Ok(())
}

/// Finish the COPY operation
pub async fn finish_copy(mut writer: Pin<Box<CopyInSink<BytesMut>>>) -> Result<u64, String> {
    writer
        .as_mut()
        .finish()
        .await
        .map_err(|e| format!("Failed to finish COPY operation: {}", e))
}
