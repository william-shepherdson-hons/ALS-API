use postgres::NoTls;

use crate::services::database::database::get_connection_string;

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Unexpected error: {0}")]
    Other(#[from] anyhow::Error),
}


pub async fn get_module_names() -> Result<Vec<String>, GeneratorError> {
    let connection_string = get_connection_string().await
        .map_err(|e| GeneratorError::Database(format!("Failed to build connection string: {e}")))?;

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| GeneratorError::Database(format!("Failed to connect to DB: {e}")))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });
    let rows = client.query("SELECT skill_name FROM SKILLS", &[])
        .await
        .map_err(|e| GeneratorError::Database(format!("Failed to fetch topics from database: {e}")))?;
    let mut topics: Vec<String> = [].to_vec();
    for row in rows {
        let topic: String = row.get(0);
        topics.push(topic);
    }
    Ok(topics)
}