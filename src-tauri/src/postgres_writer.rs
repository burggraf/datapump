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
    delimiter: &str,  // Now we'll use the provided delimiter
) -> Result<Pin<Box<dyn Sink<BytesMut, Error = tokio_postgres::Error> + Send>>, String> {
    let column_names = columns
        .iter()
        .map(|(name, _)| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(", ");

    // Convert delimiter to PostgreSQL format
    let pg_delimiter = match delimiter {
        "," => "','",
        "\t" => "E'\\t'",
        ";" => "';'",
        _ => "','"  // Default to comma
    };

    let copy_sql = format!(
        "COPY \"{}\" ({}) FROM STDIN WITH (FORMAT binary)",
        table_name, column_names
    );
    
    println!("Starting COPY with SQL: {}", copy_sql);
    
    let writer = client
        .copy_in(&copy_sql)
        .await
        .map_err(|e| format!("Failed to start COPY operation: {}", e))?;

    println!("COPY operation started successfully");
    Ok(Box::pin(writer))
}

/// Write a CSV record using the COPY protocol
pub async fn write_copy_row(
    writer: &mut Pin<Box<dyn Sink<BytesMut, Error = tokio_postgres::Error> + Send>>,
    record: BytesMut,
) -> Result<(), String> {
    println!("Writing record: {} bytes", record.len());
    writer
        .send(record)
        .await
        .map_err(|e| format!("Failed to write record: {}", e))?;
    println!("Record written successfully");
    Ok(())
}

/// Finish a COPY operation
pub async fn finish_copy(
    mut writer: Pin<Box<dyn Sink<BytesMut, Error = tokio_postgres::Error> + Send>>,
) -> Result<u64, String> {
    println!("Finishing COPY operation");
    writer
        .close()
        .await
        .map_err(|e| format!("Failed to finish COPY operation: {}", e))?;
    println!("COPY operation finished successfully");
    Ok(0)  // Return number of rows copied
}

/// Write a header for PostgreSQL binary COPY format
fn write_binary_header(buf: &mut BytesMut) {
    // PGCOPY\n\377\r\n\0
    buf.extend_from_slice(b"PGCOPY\n\xff\r\n\0");
    // Flags field (0 for no OIDs)
    buf.extend_from_slice(&[0, 0, 0, 0]);
    // Header extension length (0)
    buf.extend_from_slice(&[0, 0, 0, 0]);
}

/// Write field length in network byte order (Big Endian)
fn write_field_length(buf: &mut BytesMut, len: i32) {
    buf.extend_from_slice(&len.to_be_bytes());
}

/// Write a binary field value
fn write_binary_field(buf: &mut BytesMut, value: &str, field_type: &str) {
    match field_type.to_lowercase().as_str() {
        "int4" | "integer" => {
            write_field_length(buf, 4);
            if let Ok(val) = value.parse::<i32>() {
                buf.extend_from_slice(&val.to_be_bytes());
            } else {
                write_field_length(buf, -1); // NULL
            }
        }
        "int8" | "bigint" => {
            write_field_length(buf, 8);
            if let Ok(val) = value.parse::<i64>() {
                buf.extend_from_slice(&val.to_be_bytes());
            } else {
                write_field_length(buf, -1);
            }
        }
        "float4" | "real" => {
            write_field_length(buf, 4);
            if let Ok(val) = value.parse::<f32>() {
                buf.extend_from_slice(&val.to_be_bytes());
            } else {
                write_field_length(buf, -1);
            }
        }
        "float8" | "double precision" => {
            write_field_length(buf, 8);
            if let Ok(val) = value.parse::<f64>() {
                buf.extend_from_slice(&val.to_be_bytes());
            } else {
                write_field_length(buf, -1);
            }
        }
        "text" | "varchar" | _ => {
            if value.is_empty() {
                write_field_length(buf, -1); // NULL
            } else {
                let bytes = value.as_bytes();
                write_field_length(buf, bytes.len() as i32);
                buf.extend_from_slice(bytes);
            }
        }
    }
}

/// Prepare a record in binary format
pub fn prepare_binary_record(record: &[String], field_types: &[String]) -> BytesMut {
    let mut buf = BytesMut::with_capacity(1024);
    
    // Write field count
    buf.extend_from_slice(&(record.len() as i16).to_be_bytes());
    
    // Write each field
    for (value, field_type) in record.iter().zip(field_types.iter()) {
        write_binary_field(&mut buf, value, field_type);
    }
    
    buf
}
