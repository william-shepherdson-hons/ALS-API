use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use crate::{services::database::account::AccountError, structs::claims::Claims};
use chrono::Utc;

pub fn issue_access_token(username: &str,user_id: i32,secret: &str) -> Result<String, AccountError> {
    let now = Utc::now().timestamp() as usize;
    let exp = now + (15 * 60);

    let claims = Claims {
        sub: username.to_owned(),
        uid: user_id,
        iat: now,
        exp,
        iss: "knowledge tracing api".to_string(),
        aud: "adapt math desktop-app".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AccountError::TokenCreation(e.to_string()))
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, AccountError> {
    let validation = Validation::default();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AccountError::InvalidToken(e.to_string()))
}
