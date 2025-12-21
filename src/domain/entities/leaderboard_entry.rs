use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct LeaderboardEntry {
    pub public_key: String,
    pub total_score: i32,
    pub rank: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntryDto {
    pub public_key: String,
    pub total_score: i32,
    pub rank: u32,
}

impl From<LeaderboardEntry> for LeaderboardEntryDto {
    fn from(entry: LeaderboardEntry) -> Self {
        Self {
            public_key: entry.public_key,
            total_score: entry.total_score,
            rank: entry.rank,
        }
    }
}
