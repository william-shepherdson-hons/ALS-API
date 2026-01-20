use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SkillProgression {
    pub skill_name: String,
    pub progression: f64,
}
