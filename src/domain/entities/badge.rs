use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Badge {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub score: i32,
    pub is_unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub badge_group_id: i32,
}

#[derive(Debug, Clone)]
pub struct BadgeDto {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub score: i32,
    pub is_unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub badge_group_id: i32,
}

impl From<Badge> for BadgeDto {
    fn from(badge: Badge) -> Self {
        Self {
            id: badge.id,
            title: badge.title,
            description: badge.description,
            score: badge.score,
            is_unlocked: badge.is_unlocked,
            unlocked_at: badge.unlocked_at,
            created_at: badge.created_at,
            badge_group_id: badge.badge_group_id,
        }
    }
}
