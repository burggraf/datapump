// src/postgres_write.rs

use csv::StringRecord;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_postgres::{types::ToSql, Client, NoTls, Statement};
use bytes::BufMut;
use bytes::BytesMut;
use futures_util::sink::SinkExt;
use std::pin::Pin;

/// Open a PostgreSQL connection
pub async fn open_connection(connection_string: &str) -> Result<Client, String> {
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
        .await
        .map_err(|e| e.to_string())?;

    // Spawn the connection object to run in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
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

/// Prepare the INSERT statement for the given columns
pub async fn prepare_insert(
    client: &Client,
    table_name: &str,
    columns: &[(String, String)],
) -> Result<Statement, String> {
    let placeholders = columns
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(",");
    let column_names = columns
        .iter()
        .map(|(name, _)| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(",");

    let insert_sql = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({})",
        table_name, column_names, placeholders
    );

    client
        .prepare(&insert_sql)
        .await
        .map_err(|e| format!("Failed to prepare insert statement: {}", e))
}

/// Insert a single record with proper type conversion
pub async fn insert_record(
    statement: &Statement,
    client: &Client,
    record: &StringRecord,
    columns: &[(String, String)],
) -> Result<(), String> {
    // Add + Send here
    let mut values: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();

    for (i, (field, (_, col_type))) in record.iter().zip(columns.iter()).enumerate() {
        // Add + Send to the boxed type
        let value: Box<dyn ToSql + Sync + Send> = match col_type.to_lowercase().as_str() {
            "integer" | "int" => {
                let value = field
                    .parse::<i32>()
                    .map_err(|e| format!("Failed to parse integer at column {}: {}", i + 1, e))?;
                Box::new(value)
            }
            "float" | "float4" | "real" => {
                let value = field
                    .parse::<f32>()
                    .map_err(|e| format!("Failed to parse float at column {}: {}", i + 1, e))?;
                Box::new(value)
            }
            "float8" | "double precision" => {
                let value = field
                    .parse::<f64>()
                    .map_err(|e| format!("Failed to parse double at column {}: {}", i + 1, e))?;
                Box::new(value)
            }
            "boolean" | "bool" => {
                let value = field
                    .parse::<bool>()
                    .map_err(|e| format!("Failed to parse boolean at column {}: {}", i + 1, e))?;
                Box::new(value)
            }
            _ => Box::new(field.to_string()),
        };
        values.push(value);
    }

    let mut temp_params = Vec::with_capacity(values.len());
    for value in &values {
        // It's okay to reference as &dyn ToSql + Sync; the important part is that the Box itself is Send.
        let v: &(dyn ToSql + Sync) = value.as_ref();
        temp_params.push(v);
    }
    let params: &[&(dyn ToSql + Sync)] = &temp_params;

    client
        .execute(statement, params)
        .await
        .map_err(|e| format!("Failed to insert record: {}", e))?;

    Ok(())
}

/// Start a COPY operation for bulk loading
pub async fn start_copy(
    client: &Client,
    table_name: &str,
    columns: &[(String, String)],
) -> Result<Pin<Box<tokio_postgres::CopyInSink<BytesMut>>>, String> {
    let column_names = columns
        .iter()
        .map(|(name, _)| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(",");

    let copy_sql = format!(
        "COPY \"{}\" ({}) FROM STDIN WITH (FORMAT csv)",
        table_name, column_names
    );

    let sink = client
        .copy_in(&copy_sql)
        .await
        .map_err(|e| format!("Failed to start COPY operation: {}", e))?;
    
    Ok(Box::pin(sink))
}

/// Write a CSV record using the COPY protocol
pub async fn copy_record(
    writer: &mut Pin<Box<tokio_postgres::CopyInSink<BytesMut>>>,
    record: &csv::StringRecord,
) -> Result<(), String> {
    let mut buf = BytesMut::new();
    let csv_line = record.iter().collect::<Vec<&str>>().join(",");
    buf.put_slice(format!("{}\n", csv_line).as_bytes());
    
    writer.as_mut()
        .send(buf)
        .await
        .map_err(|e| format!("Failed to write record: {}", e))?;
    Ok(())
}

/// Finish the COPY operation
pub async fn finish_copy(mut writer: Pin<Box<tokio_postgres::CopyInSink<BytesMut>>>) -> Result<u64, String> {
    writer.as_mut()
        .finish()
        .await
        .map_err(|e| format!("Failed to finish COPY operation: {}", e))
}

/// Helper to execute a closure with retries on connection errors
async fn execute_with_retry<F, T>(f: F) -> Result<T, String>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>,
{
    let mut retries = 5;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(_e) if retries > 0 => {
                retries -= 1;
                sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Begin a transaction
pub async fn begin_transaction(client: Arc<Client>) -> Result<(), String> {
    execute_with_retry(move || {
        let client = client.clone();
        Box::pin(async move {
            client
                .execute("BEGIN", &[])
                .await
                .map_err(|e| format!("Failed to start transaction: {}", e))?;
            Ok(())
        })
    })
    .await
}

/// Commit the current transaction
pub async fn commit_transaction(client: &Client) -> Result<(), String> {
    client
        .execute("COMMIT", &[])
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to COMMIT: {}", e))
}

/// Rollback the current transaction
pub async fn rollback_transaction(client: &Client) -> Result<(), String> {
    client
        .execute("ROLLBACK", &[])
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to ROLLBACK: {}", e))
}

/// Commit the current transaction and begin a new one
pub async fn commit_and_begin_new_transaction(client: Arc<Client>) -> Result<(), String> {
    execute_with_retry(move || {
        let client = Arc::clone(&client);
        Box::pin(async move {
            client
                .execute("COMMIT", &[])
                .await
                .map_err(|e| e.to_string())?;
            client
                .execute("BEGIN", &[])
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    })
    .await
}
