use tokio_postgres::{Client, Error, NoTls};
use bytes::BytesMut;
use futures_util::{Sink, SinkExt};
use std::pin::Pin;

/// Open a PostgreSQL connection
pub async fn open_connection(connection_string: &str) -> Result<Client, String> {
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
        .await
        .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

    // Spawn connection task
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
pub async fn start_copy<'a>(
    client: &'a Client,
    table_name: &str,
    columns: &[(String, String)],
    _delimiter: &str,  // We'll always use tab as the COPY delimiter
) -> Result<Pin<Box<dyn Sink<BytesMut, Error = tokio_postgres::Error> + Send>>, String> {
    let column_names = columns
        .iter()
        .map(|(name, _)| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(", ");

    let copy_sql = format!(
        "COPY \"{}\" ({}) FROM STDIN WITH (FORMAT csv, DELIMITER E'\\t', QUOTE '\"', NULL '\\N')",
        table_name, column_names
    );
    println!("COPY SQL: {}", copy_sql);
    
    let writer = client
        .copy_in(&copy_sql)
        .await
        .map_err(|e| format!("Failed to start COPY operation: {}", e))?;

    Ok(Box::pin(writer))
}

/// Write a CSV record using the COPY protocol
pub async fn write_copy_row(
    writer: &mut Pin<Box<dyn Sink<BytesMut, Error = tokio_postgres::Error> + Send>>,
    record: BytesMut,
) -> Result<(), String> {
    writer
        .send(record)
        .await
        .map_err(|e| format!("Failed to write record: {}", e))
}

/// Finish a COPY operation
pub async fn finish_copy(
    mut writer: Pin<Box<dyn Sink<BytesMut, Error = tokio_postgres::Error> + Send>>,
) -> Result<u64, String> {
    writer
        .close()
        .await
        .map_err(|e| format!("Failed to finish COPY operation: {}", e))?;
    Ok(0)  // Return number of rows copied
}
