use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Badge {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub score: i32,
    pub is_active: bool,
    pub earned_at: Option<DateTime<Utc>>,
}
