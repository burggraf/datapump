// src/postgres_write.rs

use csv::StringRecord;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_postgres::{types::ToSql, Client, NoTls, Statement};

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

/// Insert a single record
pub async fn insert_record(
    statement: &Statement,
    client: &Client,
    record: &StringRecord,
) -> Result<(), String> {
    let params: Vec<&str> = record.iter().collect();
    let params: Vec<&(dyn ToSql + Sync)> =
        params.iter().map(|s| s as &(dyn ToSql + Sync)).collect();

    client
        .execute(statement, params.as_slice())
        .await
        .map_err(|e| format!("Failed to insert record: {}", e))?;

    Ok(())
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
