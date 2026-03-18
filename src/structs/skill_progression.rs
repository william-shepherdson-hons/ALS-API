use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SkillProgression {
    pub skill_name: String,
    pub progression: f64,
}

#[derive(Serialize, ToSchema)]
pub struct SkillProgressionWithDate {
    pub skill_name: String,
    pub progression: f64,
    pub recorded_at: String,
}