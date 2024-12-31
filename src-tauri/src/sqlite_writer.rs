// src/sqlite_writer.rs

use sqlite::{Connection, Statement};
use std::thread;
use std::time::Duration;

/// Open the SQLite database and enable WAL + NORMAL synchronous mode.
pub fn open_connection(db_path: &str) -> Result<Connection, String> {
    let connection = sqlite::open(db_path).map_err(|e| e.to_string())?;

    // WAL mode
    connection
        .execute("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

    // Synchronous = NORMAL
    connection
        .execute("PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("Failed to set synchronous mode: {}", e))?;

    Ok(connection)
}

/// Create table if not exists, given a table name and columns.
pub fn create_table(
    connection: &Connection,
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

    connection
        .execute(&create_table_sql)
        .map_err(|e| format!("Failed to create table: {}", e))?;

    Ok(())
}

/// Prepare the INSERT statement for the given columns.
pub fn prepare_insert<'a>(
    connection: &'a Connection,
    table_name: &'a str,
    columns: &'a [(String, String)],
) -> Result<Statement<'a>, String> {
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

    connection.prepare(&insert_sql).map_err(|e| {
        let msg = format!("Failed to prepare insert statement: {}", e);
        msg
    })
}

/// Actually bind the CSV record's fields into the prepared statement
/// and execute the insertion for a single record.
pub fn insert_record<'a>(
    statement: &mut Statement<'a>,
    record: &csv::StringRecord,
) -> Result<(), String> {
    // Bind each field
    for (i, field) in record.iter().enumerate() {
        if field.is_empty() {
            statement.bind((i + 1, ())).map_err(|e| e.to_string())?;
        } else {
            statement.bind((i + 1, field)).map_err(|e| e.to_string())?;
        }
    }

    // Execute
    statement.next().map_err(|e| e.to_string())?;

    // Reset for the next row
    statement.reset().map_err(|e| e.to_string())?;

    Ok(())
}

/// Helper to execute a closure with up to 5 retries on "database is locked" errors.
fn execute_with_retry<F>(connection: &Connection, f: F) -> Result<(), String>
where
    F: Fn(&Connection) -> Result<(), String>,
{
    let mut retries = 5;
    loop {
        match f(connection) {
            Ok(_) => return Ok(()),
            Err(e) if e.contains("database is locked") && retries > 0 => {
                retries -= 1;
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Start a transaction immediately.
pub fn begin_transaction(connection: &Connection) -> Result<(), String> {
    execute_with_retry(connection, |conn| {
        conn.execute("BEGIN IMMEDIATE TRANSACTION")
            .map_err(|e| format!("Failed to start transaction: {}", e))
    })
}

/// Commit the current transaction.
pub fn commit_transaction(connection: &Connection) -> Result<(), String> {
    connection
        .execute("COMMIT")
        .map_err(|e| format!("Failed to COMMIT: {}", e))
}

/// Rollback the current transaction (used on error or cancellation).
pub fn rollback_transaction(connection: &Connection) -> Result<(), String> {
    connection
        .execute("ROLLBACK")
        .map_err(|e| format!("Failed to ROLLBACK: {}", e))
}

/// Commit the current transaction, then immediately start a new one.
/// Used for batch commits.
pub fn commit_and_begin_new_transaction(connection: &Connection) -> Result<(), String> {
    execute_with_retry(connection, |conn| {
        conn.execute("COMMIT").map_err(|e| e.to_string())?;
        conn.execute("BEGIN IMMEDIATE TRANSACTION")
            .map_err(|e| e.to_string())
    })
}
