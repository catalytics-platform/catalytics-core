#[derive(Debug)]
pub struct UserProgression {
    pub progression_event_type: String,
    pub current_progress: i32,
}

#[derive(Debug, Clone)]
pub struct UserProgressionDto {
    pub progression_event_type: String,
    pub current_progress: i32,
}

impl From<UserProgression> for UserProgressionDto {
    fn from(progression: UserProgression) -> Self {
        Self {
            progression_event_type: progression.progression_event_type,
            current_progress: progression.current_progress,
        }
    }
}
