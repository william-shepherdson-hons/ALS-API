use axum::{
    routing::get,
    Router,
    Json
};


use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;


#[tokio::main]
async fn main() {

    #[derive(OpenApi)]
    #[openapi(paths(pong, get_modules), components(schemas()), tags())]
    struct ApiDoc;

    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/ping", get(pong))
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
async fn get_modules() -> Json<Vec<String>> {
    Json(vec!["test".to_string()])
}
