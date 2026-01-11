use axum::{
    Json, Router, extract::Path, http::StatusCode, response::IntoResponse, routing::{get, patch, post}
};
use base64::Engine;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use als_api::{
    middleware::auth::AuthenticatedUser, services::database::{
        account::{AccountError, check_password, check_token, create_account}, jwt::issue_access_token, knowledge_service::{get_knowledge_score, update_knowledge_score}
    }, structs::{
        account::Account, knowledge_score_request::KnowledgeScoreRequest, 
        knowledge_score_update::KnowledgeScoreUpdate, performance_update::PerformanceUpdate, 
        sign_in::SignIn, token_validation::TokenValidation
    }
};
use als_algorithm::models::knowledge_tracing_model::calculate_mastery;

#[tokio::main]
async fn main() {
    use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
    
    #[derive(OpenApi)]
    #[openapi(
        paths(pong, skill_update, register_account, login, validate_token), 
        components(schemas()),
        modifiers(&SecurityAddon),
        tags()
    )]
    struct ApiDoc;
    
    struct SecurityAddon;

    impl utoipa::Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "bearer_auth",
                    SecurityScheme::Http(
                        HttpBuilder::new()
                            .scheme(HttpAuthScheme::Bearer)
                            .bearer_format("JWT")
                            .build()
                    ),
                )
            }
        }
    }
    
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/ping", get(pong))
        .route("/students/{studentID}/skills/{skillID}/performance", patch(skill_update))
        .route("/accounts/register", post(register_account))
        .route("/accounts/login", post(login))
        .route("/accounts/validate", post(validate_token));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/ping",
    responses(
        (status = 200, description = "Life check")
    )
)]
async fn pong() -> &'static str {
    "pong"
}

#[utoipa::path(
    patch,
    path = "/students/{studentID}/skills/{skillID}/performance",
    request_body = PerformanceUpdate,
    params(
        ("studentID" = i32, Path, description = "ID of the student"),
        ("skillID" = i32, Path, description = "ID of the skill")
    ),
    responses(
        (status = 200, description = "Student Knowledge Update", body = f64),
        (status = 400, description = "Bad request")
    )
)]
async fn skill_update(
    _auth: AuthenticatedUser,
    Path((student_id, skill_id)): Path<(i32, i32)>, 
    Json(body): Json<PerformanceUpdate>
) -> impl IntoResponse {
    let fetch_skill = KnowledgeScoreRequest {
        skill_id: skill_id,
        student_id: student_id
    };
    let existing_knowledge_score = match get_knowledge_score(fetch_skill).await {
        Ok(score) => score,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Failed to fetch skill: {e}")).into_response();
        }
    };
    let new_knowledge_score = calculate_mastery(existing_knowledge_score, 0.1, 0.1, 0.1, body.correct).await;
    let knowledge_update = KnowledgeScoreUpdate {
        skill_id: skill_id,
        student_id: student_id,
        score: new_knowledge_score
    };
    let _ = match update_knowledge_score(knowledge_update).await {
        Ok(_) => (),
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Failed to update skill: {e}")).into_response();
        }
    };
    Json(new_knowledge_score).into_response()
}

#[utoipa::path(
    post,
    path = "/accounts/register",
    request_body = Account,
    responses(
        (status = 201, description = "Account created successfully"),
        (status = 400, description = "Bad request - Account creation failed")
    )
)]
async fn register_account(Json(account): Json<Account>) -> impl IntoResponse {
    match create_account(account).await {
        Ok(_) => (StatusCode::CREATED, "Account created successfully").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, format!("Failed to create account: {e}")).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/accounts/login",
    request_body = SignIn,
    responses(
        (status = 200, description = "Login successful", body = String),
        (status = 401, description = "Unauthorized - Invalid credentials"),
        (status = 400, description = "Bad request")
    )
)]
async fn login(Json(credentials): Json<SignIn>) -> impl IntoResponse {
    match check_password(credentials).await {
        Ok(token_bytes) => {
            let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token_bytes);
            (StatusCode::OK, token).into_response()
        },
        Err(AccountError::Authentication(_)) => {
            (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
        },
        Err(e) => {
            (StatusCode::BAD_REQUEST, format!("Login failed: {e}")).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/accounts/validate",
    request_body = TokenValidation,
    responses(
        (status = 200, description = "Token valid", body = String),
        (status = 401, description = "Unauthorized - Invalid or expired token"),
        (status = 400, description = "Bad request - Invalid token format")
    )
)]
async fn validate_token(Json(token_data): Json<TokenValidation>) -> impl IntoResponse {
    let token_bytes = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&token_data.token) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid token format").into_response();
        }
    };

    let token_array: [u8; 32] = match token_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid token length").into_response();
        }
    };


    match check_token(token_array).await {
        Ok(user_id) => {
            let jwt_secret = match std::env::var("JWT_SECRET") {
                Ok(secret) => secret,
                Err(_) => {
                    return (StatusCode::SERVICE_UNAVAILABLE, "JWT Token not set").into_response();
                }
            };
            let token = match issue_access_token(user_id.parse::<i32>().unwrap(), &jwt_secret) {
                Ok(token) => token,
                Err(_) => {
                    return (StatusCode::BAD_REQUEST, "Failed to issue token").into_response();
                }
            };
            (StatusCode::OK, Json(serde_json::json!({
                "valid": true,
                "user_id": user_id,
                "jwt_token": token
            }))).into_response()
        },
        Err(AccountError::Authentication(_)) => {
            (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
        },
        Err(e) => {
            (StatusCode::BAD_REQUEST, format!("Validation failed: {e}")).into_response()
        }
    }
}

