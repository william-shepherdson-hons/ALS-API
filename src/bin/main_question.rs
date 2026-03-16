use als_api::{
    enums::difficulty::Difficulty,
    middleware::auth::AuthenticatedUser,
    services::{
        database::{
            knowledge_service::{get_knowledge_score, get_skill_id},
            question_service::get_module_names,
        },
        generator::modules::{
            fetch_module_list,
            generate_questions,
            generate_word_questions,
        },
    },
    structs::{
        knowledge_score_request::KnowledgeScoreRequest,
        question_pair::QuestionPair,
    },
};

use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use reqwest::StatusCode;

use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use utoipa_swagger_ui::SwaggerUi;

use serde::Deserialize;

#[derive(Deserialize)]
struct GenerateQuery {
    amount: Option<usize>,
}

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            pong,
            get_internal_modules,
            get_modules,
            generate,
            generate_word
        ),
        components(schemas(QuestionPair)),
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
                            .build(),
                    ),
                );
            }
        }
    }

    let app = Router::new()
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .route("/ping", get(pong))
        .route("/generate/{module}", get(generate))
        .route("/generate_word/{module}", get(generate_word))
        .route("/internal_modules", get(get_internal_modules))
        .route("/modules", get(get_modules));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

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
    path = "/internal_modules",
    responses(
        (status = 200, description = "List of internal modules", body = [String])
    ),
    security(("bearer_auth" = []))
)]
async fn get_internal_modules(
    _auth: AuthenticatedUser,
) -> impl IntoResponse {
    match fetch_module_list().await {
        Ok(modules) => Json(modules).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to fetch module list: {}", e),
        ).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/modules",
    responses(
        (status = 200, description = "List of modules", body = [String])
    ),
    security(("bearer_auth" = []))
)]
async fn get_modules(
    _auth: AuthenticatedUser,
) -> impl IntoResponse {
    match get_module_names().await {
        Ok(modules) => Json(modules).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to fetch module list: {}", e),
        ).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/generate/{module}",
    params(
        ("module" = String, Path, description = "Skill name"),
        ("amount" = Option<usize>, Query, description = "Number of questions")
    ),
    responses(
        (status = 200, description = "Generated questions", body = [QuestionPair]),
        (status = 503, description = "Generator service unavailable")
    ),
    security(("bearer_auth" = []))
)]
async fn generate(
    auth: AuthenticatedUser,
    Path(module): Path<String>,
    Query(query): Query<GenerateQuery>,
) -> impl IntoResponse {

    let amount = query.amount.unwrap_or(1).min(50);

    let skill_id = match get_skill_id(&module).await {
        Ok(skill) => skill,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Failed to fetch skill id: {}", e),
            ).into_response();
        }
    };

    let student_id = auth.claims.uid;

    let progression = match get_knowledge_score(
        KnowledgeScoreRequest { skill_id, student_id }
    ).await {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Failed to fetch progression: {}", e),
            ).into_response();
        }
    };

    let difficulty = match progression {
        x if x < 0.33 => Difficulty::Easy,
        x if x < 0.66 => Difficulty::Medium,
        _ => Difficulty::Hard,
    };

    match generate_questions(module, difficulty, amount).await {
        Ok(questions) => Json(questions).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to generate questions: {}", e),
        ).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/generate_word/{module}",
    params(
        ("module" = String, Path, description = "Skill name"),
        ("amount" = Option<usize>, Query, description = "Number of questions")
    ),
    responses(
        (status = 200, description = "Generated word questions", body = [QuestionPair]),
        (status = 503, description = "Generator service unavailable")
    ),
    security(("bearer_auth" = []))
)]
async fn generate_word(
    auth: AuthenticatedUser,
    Path(module): Path<String>,
    Query(query): Query<GenerateQuery>,
) -> impl IntoResponse {

    let amount = query.amount.unwrap_or(1).min(50);

    let skill_id = match get_skill_id(&module).await {
        Ok(skill) => skill,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Failed to fetch skill id: {}", e),
            ).into_response();
        }
    };

    let student_id = auth.claims.uid;

    let progression = match get_knowledge_score(
        KnowledgeScoreRequest { skill_id, student_id }
    ).await {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Failed to fetch progression: {}", e),
            ).into_response();
        }
    };

    let difficulty = match progression {
        x if x < 0.33 => Difficulty::Easy,
        x if x < 0.66 => Difficulty::Medium,
        _ => Difficulty::Hard,
    };

    match generate_word_questions(module, difficulty, amount).await {
        Ok(questions) => Json(questions).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to generate word questions: {}", e),
        ).into_response(),
    }
}