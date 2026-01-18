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
    pub previous_rank: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Database row for real-time leaderboard calculation using CTE and window functions.
/// Fields are Optional because SQLx cannot guarantee non-nullability with complex queries,
/// even though logically these should always have values for existing users.
#[derive(sqlx::FromRow, Debug)]
struct RealtimeLeaderboardEntryDb {
    pub public_key: Option<String>,
    pub total_score: Option<i32>,
    pub rank: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
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
            "SELECT public_key, total_score, rank, previous_rank, created_at
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
                previous_rank: entry.previous_rank.map(|r| r as u32),
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
            "SELECT public_key, total_score, rank, previous_rank, created_at
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
            previous_rank: entry.previous_rank.map(|r| r as u32),
            created_at: entry.created_at,
        }))
    }

    async fn get_user_realtime_entry(
        &self,
        public_key: &str,
    ) -> AppResult<Option<LeaderboardEntry>> {
        let result = sqlx::query_as!(
            RealtimeLeaderboardEntryDb,
            r#"
            WITH ranked_users AS (
                SELECT 
                    ba.public_key,
                    ba.created_at,
                    COALESCE(SUM(b.score), 0)::INTEGER as total_score,
                    ROW_NUMBER() OVER (ORDER BY COALESCE(SUM(b.score), 0) DESC, ba.created_at ASC) as rank
                FROM beta_applicants ba
                LEFT JOIN beta_applicant_badges bab ON ba.id = bab.beta_applicant_id  
                LEFT JOIN badges b ON bab.badge_id = b.id
                GROUP BY ba.id, ba.public_key, ba.created_at
            )
            SELECT public_key, created_at, total_score, rank 
            FROM ranked_users 
            WHERE public_key = $1
            "#,
            public_key
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(result.map(|entry| LeaderboardEntry {
            public_key: entry.public_key.unwrap_or_default(),
            total_score: entry.total_score.unwrap_or(0),
            rank: entry.rank.unwrap_or(0) as u32,
            previous_rank: None, // Real-time queries don't have previous rank data
            created_at: entry.created_at.unwrap_or_else(|| chrono::Utc::now()),
        }))
    }

    async fn add_new_user_to_leaderboard(
        &self,
        beta_applicant_id: i32,
        public_key: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO leaderboard_entries 
             (beta_applicant_id, public_key, total_score, rank, previous_rank, created_at, updated_at)
             VALUES ($1, $2, 0, (SELECT COALESCE(MAX(rank), 0) + 1 FROM leaderboard_entries), NULL, NOW(), NOW())",
            beta_applicant_id,
            public_key
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }
}
