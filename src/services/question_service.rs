use crate::structs::{knowledge_score_request::KnowledgeScoreRequest, knowledge_score_update::KnowledgeScoreUpdate};

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
    Ok(0.1)
}

pub async fn update_knowledge_score(knowledge_update: KnowledgeScoreUpdate) -> Result<i32, KnowledgeError> {
    Ok(1)
}