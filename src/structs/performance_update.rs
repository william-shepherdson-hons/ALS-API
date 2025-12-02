use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct PerformanceUpdate {
    pub correct: bool
}