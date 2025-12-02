use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug)]
pub struct BetaApplicantBadgeDb {
    pub badge_id: i32,
    pub created_at: DateTime<Utc>,
}
