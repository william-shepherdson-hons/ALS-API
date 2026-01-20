use tokio_postgres::NoTls;
use crate::{services::database::database::get_connection_string, structs::{knowledge_score_request::KnowledgeScoreRequest, knowledge_score_update::KnowledgeScoreUpdate}};

#[derive(thiserror::Error, Debug)]
pub enum KnowledgeError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid knowledge score update: {0}")]
    InvalidInput(String),

    #[error("Unexpected error: {0}")]
    Other(#[from] anyhow::Error),
}

pub async fn get_knowledge_score(skill_request: KnowledgeScoreRequest) -> Result<f64, KnowledgeError> {
    let connection_string = get_connection_string().await
        .map_err(|e| KnowledgeError::Database(format!("Failed to build connection string: {e}")))?;

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| KnowledgeError::Database(format!("Failed to connect to DB: {e}")))?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });

    let row = client.query_one("SELECT progression FROM progression WHERE user_id=$1 AND skill_id=$2",&[&skill_request.student_id, &skill_request.skill_id])
        .await
        .map_err(|e| KnowledgeError::Database(format!("Failed to get knowledge score: {e}")))?;

    Ok(row.get(0))
}

pub async fn update_knowledge_score(update: KnowledgeScoreUpdate) -> Result<(), KnowledgeError> {
    let connection_string = get_connection_string().await
        .map_err(|e| KnowledgeError::Database(format!("Failed to build connection string: {e}")))?;

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| KnowledgeError::Database(format!("Failed to connect to DB: {e}")))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });

    client.execute("UPDATE progression SET progression = $1 WHERE user_id=$2 AND skill_id=$3",&[&update.score, &update.student_id, &update.skill_id])
        .await
        .map_err(|e| KnowledgeError::Database(format!("Failed to update score: {e}")))?;

    Ok(())
}

pub async fn get_skill_id(skill_name: &str) -> Result<i32, KnowledgeError> {
    let connection_string = get_connection_string().await
        .map_err(|e| KnowledgeError::Database(format!("Failed to build connection string: {e}")))?;

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| KnowledgeError::Database(format!("Failed to connect to DB: {e}")))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });
    let row = client.query_one("SELECT skill_id FROM SKILLS WHERE skill_name=$1", &[&skill_name])
        .await
        .map_err(|e| KnowledgeError::Database(format!("Failed to fetch skill id: {e}")))?;

    Ok(row.get(0))
}
