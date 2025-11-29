use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct BetaApplicant {
    pub id: u32,
    pub public_key: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}