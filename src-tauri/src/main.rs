// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod csv_schema;
mod csv_to_sqlite;
mod postgres;

use postgres::QueryResult;

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

#[macro_use]
mod flat_files;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            postgres::execute_postgres_query,
            execute_sqlite_query,
            flat_files::get_real_path,
            flat_files::append_to_file,
            csv_to_sqlite::get_csv_schema,
            csv_to_sqlite::csv_to_sqlite,
            csv_to_sqlite::cancel_migration
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
