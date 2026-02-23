#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = reqwest::Client::new();

    let res: serde_json::Value = client
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "gpt-5-nano",
            "input": "Hello!"
        }))
        .send()
        .await?
        .json()
        .await?;

    println!("{}", res);

    Ok(())
}