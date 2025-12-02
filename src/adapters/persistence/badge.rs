use crate::adapters::persistence::PostgresPersistence;
use crate::adapters::persistence::beta_applicant_badge::BetaApplicantBadgeDb;
use crate::app_error::{AppError, AppResult};
use crate::entities::badge::Badge;
use crate::use_cases::badge::BadgePersistence;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(sqlx::FromRow, Debug)]
pub struct BadgeDb {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub score: i32,
}

impl PostgresPersistence {
    fn convert_to_badges(
        &self,
        badges: Vec<BadgeDb>,
        badges_earned: Vec<BetaApplicantBadgeDb>,
    ) -> AppResult<Vec<Badge>> {
        let earned_map = badges_earned
            .into_iter()
            .map(|badge| (badge.badge_id, badge))
            .collect::<HashMap<_, _>>();

        let result = badges.into_iter().map(|badge| Badge {
            id: badge.id,
            title: badge.title,
            description: badge.description,
            score: badge.score,
            is_active: earned_map.get(&badge.id).is_some(),
            earned_at: earned_map.get(&badge.id).map(|badge| badge.created_at),
        });

        Ok(result.collect::<Vec<Badge>>())
    }
}

#[async_trait]
impl BadgePersistence for PostgresPersistence {
    async fn read_badges(&self, public_key: &str) -> AppResult<Vec<Badge>> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;
        let badges = sqlx::query_as!(BadgeDb, "SELECT * FROM badges")
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::from)?;

        let badges_earned = sqlx::query_as!(
            BetaApplicantBadgeDb,
            "SELECT badge_id, created_at FROM beta_applicant_badges WHERE beta_applicant_id = $1",
            applicant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(self.convert_to_badges(badges, badges_earned)?)
    }
}
