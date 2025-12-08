use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Badge {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub score: i32,
    pub is_unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
}
