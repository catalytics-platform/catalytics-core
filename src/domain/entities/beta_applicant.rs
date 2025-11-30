use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct BetaApplicant {
    pub id: i32,
    pub public_key: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub referral_code: String,
    pub referred_by: Option<String>,
}
