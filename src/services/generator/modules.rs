use reqwest::Client;
use serde::Deserialize;

use crate::{enums::difficulty::Difficulty, helpers::topic_conversion::skill_name_to_api_string, structs::{module_list::ModuleList, question_pair::{QuestionPair}}};

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error("Connection to generator error: {0}")]
    Connection(String),
    #[error("ChatGPT error: {0}")]
    GPT(String),
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
    let module = skill_name_to_api_string(&module).unwrap_or("error");
    let response = client
        .get("http://172.18.0.12:5000/generate")
        .query(&[
            ("filter", module),
            ("difficulty", &difficulty.to_string())
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

pub async fn generate_word_question(module: String, difficulty: Difficulty) -> Result<QuestionPair, GeneratorError> {

    let mut question_pair = generate_question(module, difficulty).await?;

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| GeneratorError::GPT("Failed to fetch API Key".into()))?;

    let client = reqwest::Client::new();

    let prompt = format!(
        "Take the question below and convert it into a word question \
        which makes it easier to understand. \
        Output HTML only. Do not include explanation. \
        Keep the answer identical.\n\n{}",
        question_pair.question
    );

    let response = client
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "gpt-5-nano",
            "input": prompt
        }))
        .send()
        .await
        .map_err(|e| GeneratorError::GPT(format!("Request failed: {e}")))?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();

        return Err(GeneratorError::GPT(format!(
            "OpenAI returned {}: {}",
            status, body
        )));
    }

    let res: serde_json::Value = response
        .json()
        .await
        .map_err(|e| GeneratorError::GPT(format!("JSON parse error: {e}")))?;


    let output_text = res["output_text"]
        .as_str()
        .ok_or_else(|| GeneratorError::GPT(format!(
            "Missing output_text. Full response: {:#?}",
            res
        )))?
        .to_string();

    question_pair.question = output_text;

    Ok(question_pair)
}