use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use crate::{
    services::database::jwt::validate_jwt,
    structs::claims::Claims,
};

pub struct AuthenticatedUser {
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT secret not configured".to_string()))?;

        let claims = validate_jwt(bearer.token(), &jwt_secret)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid or expired token {e}")))?;

        Ok(Self { claims })
    }
}