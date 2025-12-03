use crate::structs::module_list::ModuleList;

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