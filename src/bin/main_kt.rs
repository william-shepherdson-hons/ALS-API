use axum::{
    Json, Router, extract::Path, http::StatusCode, response::IntoResponse, routing::{get, patch, post}
};
use base64::Engine;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use als_api::{
    services::database::{
        account::{AccountError, check_password, create_account}, knowledge_service::{get_knowledge_score, update_knowledge_score}
    }, 
    structs::{
        account::Account, knowledge_score_request::KnowledgeScoreRequest, knowledge_score_update::KnowledgeScoreUpdate, performance_update::PerformanceUpdate, sign_in::SignIn
    }
};
use als_algorithm::models::knowledge_tracing_model::calculate_mastery;

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(
        paths(pong, skill_update, register_account, login), 
        components(schemas()), 
        tags()
    )]
    struct ApiDoc;
    
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/ping", get(pong))
        .route("/students/{studentID}/skills/{skillID}/performance", patch(skill_update))
        .route("/accounts/register", post(register_account))
        .route("/accounts/login", post(login));
    
    // run on port 3000
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
        (status = 200, description = "Login successful", body = String), // Return the refresh token as base64
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
