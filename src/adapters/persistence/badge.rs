use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::badge::Badge;
use crate::use_cases::badge::BadgePersistence;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug)]
pub struct BadgeDb {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub score: i32,
    pub is_active: Option<bool>,
    pub earned_at: Option<DateTime<Utc>>,
}

impl PostgresPersistence {
    fn convert_to_badge(&self, db: BadgeDb) -> Badge {
        Badge {
            id: db.id,
            title: db.title,
            description: db.description,
            score: db.score,
            is_active: db.is_active.unwrap_or(false),
            earned_at: db.earned_at,
        }
    }
}

#[async_trait]
impl BadgePersistence for PostgresPersistence {
    async fn read_badges(&self, public_key: &str) -> AppResult<Vec<Badge>> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;
        let badges = sqlx::query_as!(
            BadgeDb,
            "SELECT
    b.*,
    CASE WHEN bab.badge_id IS NOT NULL THEN true ELSE false END AS is_active,
    bab.created_at AS earned_at
FROM badges b
LEFT JOIN beta_applicant_badges bab ON b.id = bab.badge_id AND bab.beta_applicant_id = $1",
            applicant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(badges
            .into_iter()
            .map(|badge_db| self.convert_to_badge(badge_db))
            .collect::<Vec<_>>())
    }
}
