use crate::{services::database::database::get_connection_string, structs::{account::Account, sign_in::SignIn}};
use base64::Engine;
use tokio_postgres::NoTls;
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use rand_core::{OsRng, RngCore};

#[derive(thiserror::Error, Debug)]
pub enum AccountError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Hashing error: {0}")]
    Hashing(String),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Unexpected error: {0}")]
    Other(#[from] anyhow::Error),
}
pub async fn create_account(new_account: Account) -> Result<(), AccountError> {
    let connection_string = get_connection_string().await
        .map_err(|e| AccountError::Database(format!("Failed to build connection string: {e}")))?;

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| AccountError::Database(format!("Failed to connect to DB: {e}")))?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(new_account.password.as_bytes(), &salt)
        .map_err(|e | AccountError::Hashing(format!("Failed to hash password: {e}")))?
        .to_string();
    client.execute("INSERT INTO USERS (first_name, last_name, username, password_hash) VALUES($1, $2, $3, $4)", &[&new_account.first_name, &new_account.last_name, &new_account.username, &hash])
        .await
        .map_err(|e| AccountError::Database(format!("Failed to insert new user: {e}")))?;

    Ok(())
}

pub async fn check_password(account_details: SignIn) -> Result<[u8; 32], AccountError> {
    let connection_string = get_connection_string().await
        .map_err(|e| AccountError::Database(format!("Failed to build connection string: {e}")))?;

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| AccountError::Database(format!("Failed to connect to DB: {e}")))?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });
    let row = client.query_one("SELECT password_hash FROM USERS WHERE username=$1 ", &[&account_details.username])
        .await
        .map_err(|e | AccountError::Database(format!("Failed to find user: {e}")))?;
    let hash: String = row.get(0);
    let parsed_hash = PasswordHash::new(&hash)
        .map_err(|e| AccountError::Hashing(format!("Failed to parse stored hash: {e}")))?;
    let argon2 = Argon2::default();

    let bytes = create_refresh_token(&account_details).await?;
    match argon2.verify_password(account_details.password.as_bytes(), &parsed_hash) {
        Ok(_) =>  Ok(bytes),
        Err(_) => Err(AccountError::Authentication(format!("Invalid account details"))),
    }
}

async fn create_refresh_token(account_details: &SignIn) -> Result<[u8; 32], AccountError> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(&bytes, &salt)
        .map_err(|e | AccountError::Hashing(format!("Failed to hash password: {e}")))?
        .to_string();


    let connection_string = get_connection_string().await
        .map_err(|e| AccountError::Database(format!("Failed to build connection string: {e}")))?;

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| AccountError::Database(format!("Failed to connect to DB: {e}")))?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });

    let row = client.query_one("SELECT user_id FROM USERS WHERE username=$1 ", &[&account_details.username])
        .await
        .map_err(|e | AccountError::Database(format!("Failed to find user: {e}")))?;
    let user_id: String = row.get(0);

    client.execute("INSERT INTO SESSIONS (user_id, refresh_token_hash) VALUES($1, $2)", &[&user_id, &hash])
        .await
        .map_err(|e| AccountError::Database(format!("Failed to insert new user: {e}")))?;

    Ok(bytes)

}