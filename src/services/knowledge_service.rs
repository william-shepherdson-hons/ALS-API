use postgres::{Client,NoTls};
use crate::{services::database::get_connection_string, structs::{knowledge_score_request::KnowledgeScoreRequest, knowledge_score_update::KnowledgeScoreUpdate}};

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
    let connection_string = match get_connection_string().await {
        Ok(connection) => connection,
        Err(e) => return Err(KnowledgeError::Database(format!("Failed to get database environment variables: {e}")))
    };

    let mut client = match Client::connect(&connection_string, NoTls) {
        Ok(client ) => client,
        Err(e) => return Err(KnowledgeError::Database(format!("Failed to create client: {}", e)))
    };

    let knowledge_score: f64 =  match client.query_one("SELECT progression FROM progression WHERE user_id=$1 AND skill_id=$2", &[&skill_request.student_id, &skill_request.skill_id]){
        Ok(row) => row.get(0),
        Err(e) => return Err(KnowledgeError::Database(format!("Failed to get knowledge score: {}", e))) 
    };

    Ok(knowledge_score)
}

pub async fn update_knowledge_score(knowledge_update: KnowledgeScoreUpdate) -> Result<(), KnowledgeError> {
    Ok(())
}