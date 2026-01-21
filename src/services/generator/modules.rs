use reqwest::Client;
use serde::Deserialize;

use crate::{enums::difficulty::Difficulty, helpers::topic_conversion::skill_name_to_api_string, structs::{module_list::ModuleList, question_pair::QuestionPair}};

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error("Connection to generator error: {0}")]
    Connection(String),
    #[error("Unexpected error: {0}")]
    Other(#[from] anyhow::Error),
}


pub async fn fetch_module_list() -> Result<Vec<String>, GeneratorError> {
    let body = reqwest::get("http://172.18.0.12:5000/modules")
        .await
        .map_err(|e| GeneratorError::Connection(format!("Failed to get a response from generator: {}", e)))?;
    let module_list: ModuleList = body.json()
        .await
        .map_err(|e| GeneratorError::Connection(format!("Failed to parse generator response: {}", e)))?;
    Ok(module_list.modules)
}


#[derive(Debug, Deserialize)]
struct GenerateResponse {
    items: Vec<QuestionPair>,
}

pub async fn generate_question(module: String, difficulty: Difficulty) -> Result<QuestionPair, GeneratorError>{
    let client = Client::new();
    let module = skill_name_to_api_string(&module);
    let response = client
        .get("http://172.18.0.12:5000/generate")
        .query(&[
            ("filter", module),
            ("difficulty", difficulty.to_string())
        ])
        .send()
        .await
        .map_err(|e| GeneratorError::Connection(format!("Failed to get a response from generator: {}", e)))?;
    let body: GenerateResponse = response
            .json()
            .await
            .map_err(|e| GeneratorError::Connection(format!("Failed to parse generator response: {}", e)))?;
    let result = body.items
            .into_iter()
            .next()
            .ok_or_else(|| GeneratorError::Connection(format!("No question generated")))?;
    Ok(result)

}