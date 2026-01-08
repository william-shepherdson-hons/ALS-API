use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub struct QuestionPair {
    pub question: String,
    pub answer: String
}