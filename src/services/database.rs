use std::{env, error::Error};


pub async fn get_connection_string() -> Result<String, Box<dyn Error>> {
    let user = env::var("POSTGRES_USER")?;
    let db = env::var("POSTGRES_DB")?;
    let pass = env::var("POSTGRES_PASSWORD")?;
    let ip = env::var("POSTGRES_IP")?;
    Ok(format!("host={} user={} db={} password={}",ip,user,db,pass).to_string())
}