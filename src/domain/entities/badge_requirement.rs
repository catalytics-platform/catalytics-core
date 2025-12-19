#[derive(Debug)]
pub struct BadgeRequirement {
    pub badge_id: i32,
    pub progression_event_type: String,
    pub operation: String,
    pub required_count: i32,
}

#[derive(Debug, Clone)]
pub struct BadgeRequirementDto {
    pub badge_id: i32,
    pub progression_event_type: String,
    pub operation: String,
    pub required_count: i32,
}

impl From<BadgeRequirement> for BadgeRequirementDto {
    fn from(requirement: BadgeRequirement) -> Self {
        Self {
            badge_id: requirement.badge_id,
            progression_event_type: requirement.progression_event_type,
            operation: requirement.operation,
            required_count: requirement.required_count,
        }
    }
}
