use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct BadgeGroup {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}
