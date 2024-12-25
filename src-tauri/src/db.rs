use tokio_postgres::{Client, NoTls, Error};

pub async fn connect_db(url: &str) -> Result<Client, Error> {
    let (client, connection) =
        tokio_postgres::connect(url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

#[tauri::command]
pub async fn test_database_connection(connection_string: String) -> Result<String, String> {
    match connect_db(&connection_string).await {
        Ok(client) => {
            match client.query_one("SELECT version()", &[]).await {
                Ok(row) => {
                    let version: String = row.get(0);
                    Ok(version)
                }
                Err(e) => Err(format!("Failed to execute query: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to connect to database: {}", e)),
    }
}