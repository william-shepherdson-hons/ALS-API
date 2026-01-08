use std::fmt;
use utoipa::ToSchema;
#[derive(Debug, Clone, Copy, ToSchema)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard
}
impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        };
        write!(f, "{}", s)
    }
}



