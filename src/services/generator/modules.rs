use reqwest::Client;
use serde::Deserialize;

use crate::{
    enums::difficulty::Difficulty,
    helpers::topic_conversion::skill_name_to_api_string,
    structs::{
        module_list::ModuleList,
        question_pair::QuestionPair
    }
};

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error("Connection to generator error: {0}")]
    Connection(String),

    #[error("ChatGPT error: {0}")]
    GPT(String),

    #[error("Unexpected error: {0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    items: Vec<QuestionPair>,
}

pub async fn fetch_module_list() -> Result<Vec<String>, GeneratorError> {
    let body = reqwest::get("http://172.18.0.12:5000/modules")
        .await
        .map_err(|e| GeneratorError::Connection(
            format!("Failed to get a response from generator: {}", e)
        ))?;

    let module_list: ModuleList = body
        .json()
        .await
        .map_err(|e| GeneratorError::Connection(
            format!("Failed to parse generator response: {}", e)
        ))?;

    Ok(module_list.modules)
}

pub async fn generate_questions(
    module: String,
    difficulty: Difficulty,
    amount: usize,
) -> Result<Vec<QuestionPair>, GeneratorError> {

    let client = Client::new();
    let module = skill_name_to_api_string(&module).unwrap_or("error");

    let response = client
        .get("http://172.18.0.12:5000/generate")
        .query(&[
            ("filter", module),
            ("difficulty", &difficulty.to_string()),
            ("amount", &amount.to_string())
        ])
        .send()
        .await
        .map_err(|e| GeneratorError::Connection(
            format!("Failed to get a response from generator: {}", e)
        ))?;

    let body: GenerateResponse = response
        .json()
        .await
        .map_err(|e| GeneratorError::Connection(
            format!("Failed to parse generator response: {}", e)
        ))?;

    if body.items.is_empty() {
        return Err(GeneratorError::Connection("No questions generated".into()));
    }

    Ok(body.items)
}

pub async fn generate_word_questions(
    module: String,
    difficulty: Difficulty,
    amount: usize,
) -> Result<Vec<QuestionPair>, GeneratorError> {

    let mut questions = generate_questions(module, difficulty, amount).await?;

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| GeneratorError::GPT("Failed to fetch API Key".into()))?;

    let client = reqwest::Client::new();

    let question_list: Vec<String> =
        questions.iter().map(|q| q.question.clone()).collect();

    let prompt = format!(
        "Convert the following maths questions into clearer word problems.

Output JSON ONLY.

Format:
[
  {{\"question\":\"<html question 1>\"}},
  {{\"question\":\"<html question 2>\"}}
]

Rules:
- Output valid JSON only
- Preserve answers exactly
- Maintain mathematical formatting in HTML
- Use proper subscript and superscript formatting

Questions:
{:?}",
        question_list
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

    let output_text = res["output"]
        .as_array()
        .and_then(|outputs| {
            outputs.iter().find_map(|item| {
                if item["type"] == "message" {
                    item["content"]
                        .as_array()
                        .and_then(|contents| {
                            contents.iter().find_map(|c| {
                                if c["type"] == "output_text" {
                                    c["text"].as_str()
                                } else {
                                    None
                                }
                            })
                        })
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| GeneratorError::GPT("Invalid response format".into()))?;

    let transformed: Vec<serde_json::Value> =
        serde_json::from_str(output_text)
            .map_err(|e| GeneratorError::GPT(
                format!("Invalid JSON output: {e}")
            ))?;

    for (i, item) in transformed.iter().enumerate() {
        if let Some(text) = item["question"].as_str() {
            if i < questions.len() {
                questions[i].question = text.to_string();
            }
        }
    }

    Ok(questions)
}