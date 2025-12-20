use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::leaderboard_entry::LeaderboardEntry;
use crate::use_cases::leaderboard::LeaderboardPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug)]
struct LeaderboardEntryDb {
    pub public_key: String,
    pub total_score: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl PostgresPersistence {
    fn mask_public_key(public_key: &str) -> String {
        if public_key.len() <= 8 {
            public_key.to_string()
        } else {
            format!(
                "{}...{}",
                &public_key[..4],
                &public_key[public_key.len() - 4..]
            )
        }
    }
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
            r#"
            SELECT 
                ba.public_key,
                ba.created_at,
                SUM(b.score) as total_score
            FROM beta_applicants ba
            LEFT JOIN beta_applicant_badges bab ON ba.id = bab.beta_applicant_id  
            LEFT JOIN badges b ON bab.badge_id = b.id
            GROUP BY ba.id, ba.public_key, ba.created_at
            ORDER BY COALESCE(SUM(b.score), 0) DESC, ba.created_at ASC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        let result = entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| LeaderboardEntry {
                public_key: entry.public_key.clone(),
                masked_public_key: Self::mask_public_key(&entry.public_key),
                total_score: entry.total_score.unwrap_or(0) as i32,
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
}
