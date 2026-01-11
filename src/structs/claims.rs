use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub uid: i32,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
}
