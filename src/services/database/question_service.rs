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


pub fn get_modules() -> Result<Vec<String>, GeneratorError> {
    Ok(["string".to_string()].to_vec())
}