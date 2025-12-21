use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::leaderboard_entry::LeaderboardEntry;
use crate::use_cases::leaderboard::LeaderboardPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[derive(sqlx::FromRow, Debug)]
struct LeaderboardEntryDb {
    pub public_key: String,
    pub total_score: i32,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
impl LeaderboardPersistence for PostgresPersistence {
    async fn get_leaderboard_entries(
        &self,
        limit: u32,
        offset: u32,
    ) -> AppResult<Vec<LeaderboardEntry>> {
        let rows = sqlx::query(
            "SELECT id, public_key, total_score, created_at 
             FROM beta_applicants_leaderboard 
             LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        let entries: Vec<LeaderboardEntryDb> = rows
            .into_iter()
            .map(|row| LeaderboardEntryDb {
                public_key: row.get("public_key"),
                total_score: row.get("total_score"),
                created_at: row.get("created_at"),
            })
            .collect();

        let result = entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| LeaderboardEntry {
                public_key: entry.public_key,
                total_score: entry.total_score,
                rank: (offset as usize + index + 1) as u32,
                created_at: entry.created_at,
            })
            .collect();

        Ok(result)
    }

    async fn get_total_users_with_badges(&self) -> AppResult<u32> {
        let count = sqlx::query!("SELECT COUNT(DISTINCT ba.id) as count FROM beta_applicants ba")
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(count.count.unwrap_or(0) as u32)
    }

    async fn get_user_rank(&self, public_key: &str) -> AppResult<u32> {
        let result = sqlx::query!(
            r#"
            WITH ranked_users AS (
                SELECT 
                    public_key,
                    ROW_NUMBER() OVER (ORDER BY total_score DESC, created_at) as rank
                FROM beta_applicants_leaderboard
            )
            SELECT rank FROM ranked_users WHERE public_key = $1
            "#,
            public_key
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(result.map(|row| row.rank.unwrap_or(0) as u32).unwrap_or(0))
    }

    async fn get_user_leaderboard_entry(
        &self,
        public_key: &str,
    ) -> AppResult<Option<LeaderboardEntry>> {
        let result = sqlx::query!(
            r#"
            WITH ranked_users AS (
                SELECT 
                    id,
                    public_key,
                    created_at,
                    total_score,
                    ROW_NUMBER() OVER (ORDER BY total_score DESC, created_at) as rank
                FROM beta_applicants_leaderboard
            )
            SELECT id, public_key, created_at, total_score, rank 
            FROM ranked_users WHERE public_key = $1
            "#,
            public_key
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(result.map(|row| LeaderboardEntry {
            public_key: row.public_key.unwrap_or_default(),
            total_score: row.total_score.unwrap_or(0),
            rank: row.rank.unwrap_or(0) as u32,
            created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
        }))
    }
}
