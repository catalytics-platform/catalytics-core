use crate::adapters::persistence::PostgresPersistence;
use crate::adapters::persistence::beta_applicant_badge::BetaApplicantBadgeDb;
use crate::app_error::{AppError, AppResult};
use crate::entities::badge::Badge;
use crate::use_cases::badge::BadgePersistence;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(sqlx::FromRow, Debug)]
pub struct BadgeDb {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub score: i32,
    pub created_at: DateTime<Utc>,
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
            is_unlocked: earned_map.get(&badge.id).is_some(),
            unlocked_at: earned_map.get(&badge.id).map(|badge| badge.created_at),
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

    async fn create_badge(&self, public_key: &str, badge_id: i32, value: i32) -> AppResult<()> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;

        sqlx::query!(
            r#"
            INSERT INTO beta_applicant_badges (beta_applicant_id, badge_id)
            SELECT $1, bc.badge_id
            FROM badge_conditions bc
            WHERE bc.badge_id = $2
              AND (
                (bc.operation = 'eq' AND $3 = bc.required_count) OR
                (bc.operation = 'gte' AND $3 >= bc.required_count)
              )
            ON CONFLICT (beta_applicant_id, badge_id) DO NOTHING
            "#,
            applicant_id,
            badge_id,
            value
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    async fn create_catics_badges(&self, public_key: &str, catics_balance: f64) -> AppResult<()> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;
        let balance_as_int = catics_balance as i32;
        Ok(self
            .create_badges_for_progression(applicant_id, 2, balance_as_int)
            .await?)
    }

    async fn create_staked_jup_badges(
        &self,
        public_key: &str,
        staked_jup_balance: f64,
    ) -> AppResult<()> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;
        let balance_as_int = staked_jup_balance as i32;
        Ok(self
            .create_badges_for_progression(applicant_id, 5, balance_as_int)
            .await?)
    }

    // todo: remove progress_count, query db
    async fn create_badges_for_progression(
        &self,
        applicant_id: i32,
        progression_event_type_id: i32,
        progress_count: i32,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO beta_applicant_progressions (beta_applicant_id, progression_event_type_id, progress_count)
            VALUES ($1, $2, $3)
            ON CONFLICT (beta_applicant_id, progression_event_type_id)
            DO UPDATE SET progress_count = EXCLUDED.progress_count
            "#,
            applicant_id,
            progression_event_type_id,
            progress_count
        )
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;

        sqlx::query!(
            r#"
            INSERT INTO beta_applicant_badges (beta_applicant_id, badge_id)
            SELECT $1, bc.badge_id
            FROM badge_conditions bc
            WHERE bc.progression_event_type_id = $2
              AND (
                (bc.operation = 'eq' AND $3 = bc.required_count) OR
                (bc.operation = 'gte' AND $3 >= bc.required_count)
              )
            ON CONFLICT (beta_applicant_id, badge_id) DO NOTHING
            "#,
            applicant_id,
            progression_event_type_id,
            progress_count
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }
}
