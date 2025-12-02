use axum::{
    Json, Router, body, extract::Path, response::IntoResponse, routing::{get,patch}
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use als_api::structs::performance_update::PerformanceUpdate;

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(paths(pong, skill_update), components(schemas()), tags())]
    struct ApiDoc;

    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/ping", get(pong))
        .route("/students/{studentID}/skills/{skillID}/performance", patch(skill_update));

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
async fn skill_update(Path((student_id, skill_id)) : Path<(i32,i32)>, Json(body) : Json<PerformanceUpdate>) -> impl IntoResponse {
    Json(0.1)
}

