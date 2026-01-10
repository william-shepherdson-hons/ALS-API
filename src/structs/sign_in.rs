use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SignIn {
    pub username: String,
    pub password: String
}