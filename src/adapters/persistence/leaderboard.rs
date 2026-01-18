use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::leaderboard_entry::LeaderboardEntry;
use crate::use_cases::leaderboard::LeaderboardPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug)]
struct LeaderboardEntryDb {
    pub public_key: String,
    pub total_score: i32,
    pub rank: i32,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
impl LeaderboardPersistence for PostgresPersistence {
    async fn get_leaderboard_entries(
        &self,
        limit: u32,
        offset: u32,
    ) -> AppResult<Vec<LeaderboardEntry>> {
        let entries = sqlx::query_as!(
            LeaderboardEntryDb,
            "SELECT public_key, total_score, rank, created_at
             FROM leaderboard_entries 
             ORDER BY rank 
             LIMIT $1 OFFSET $2",
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        let result = entries
            .into_iter()
            .map(|entry| LeaderboardEntry {
                public_key: entry.public_key,
                total_score: entry.total_score,
                rank: entry.rank as u32,
                created_at: entry.created_at,
            })
            .collect();

        Ok(result)
    }

    async fn get_total_users_with_badges(&self) -> AppResult<u32> {
        let count = sqlx::query_scalar!("SELECT COUNT(*) as count FROM leaderboard_entries")
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(count.unwrap_or(0) as u32)
    }

    async fn get_user_rank(&self, public_key: &str) -> AppResult<u32> {
        let result = sqlx::query!(
            "SELECT rank FROM leaderboard_entries WHERE public_key = $1",
            public_key
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(result.map(|row| row.rank as u32).unwrap_or(0))
    }

    async fn get_user_leaderboard_entry(
        &self,
        public_key: &str,
    ) -> AppResult<Option<LeaderboardEntry>> {
        let result = sqlx::query_as!(
            LeaderboardEntryDb,
            "SELECT public_key, total_score, rank, created_at
             FROM leaderboard_entries 
             WHERE public_key = $1",
            public_key
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(result.map(|entry| LeaderboardEntry {
            public_key: entry.public_key,
            total_score: entry.total_score,
            rank: entry.rank as u32,
            created_at: entry.created_at,
        }))
    }
}
