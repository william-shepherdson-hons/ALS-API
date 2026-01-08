use als_api::{enums::difficulty::Difficulty, services::generator::modules::{fetch_module_list, generate_question}, structs::question_pair::QuestionPair};
use axum::{
    Json, Router, response::IntoResponse, routing::get
};
use axum::extract::Path;
use reqwest::StatusCode;
use utoipa::{OpenApi};
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(paths(pong, get_modules, generate), components(schemas(QuestionPair)), tags())]
    struct ApiDoc;
    
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/ping", get(pong))
        .route("/generate/{module}/{difficulty}", get(generate))
        .route("/modules", get(get_modules));
    
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
    get,
    path = "/modules",
    responses(
        (status = 200, description = "List of Modules", body = [String])
    )
)]
async fn get_modules() -> impl IntoResponse {
    let modules = match fetch_module_list().await {
        Ok(modules) => modules,
        Err(e) => {
            return (StatusCode::SERVICE_UNAVAILABLE, format!("Failed to fetch module list: {}", e)).into_response();
        }
    };
    Json(modules).into_response()
}

#[utoipa::path(
    get,
    path = "/generate/{module}/{difficulty}",
    params(
        ("module" = String, Path, description = "Module ID"),
        ("difficulty" = String, Path, description = "Difficulty of question")
    ),
    responses(
        (status = 200, description = "Generated question", body = QuestionPair),
        (status = 503, description = "Generator service unavailable"),
        (status = 400, description = "Invalid difficulty parameter")
    )
)]
async fn generate(Path((module, difficulty)): Path<(String, String)>) -> impl IntoResponse {
    // Parse difficulty from string
    let difficulty = match difficulty.to_lowercase().as_str() {
        "easy" => Difficulty::Easy,
        "medium" => Difficulty::Medium,
        "hard" => Difficulty::Hard,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid difficulty: {}. Must be one of: easy, medium, hard", difficulty)
            ).into_response();
        }
    };
    
    match generate_question(module, difficulty).await {
        Ok(question_pair) => Json(question_pair).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to generate question: {}", e)
        ).into_response()
    }
}