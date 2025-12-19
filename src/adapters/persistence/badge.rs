use crate::adapters::persistence::PostgresPersistence;
use crate::adapters::persistence::beta_applicant_badge::BetaApplicantBadgeDb;
use crate::app_error::{AppError, AppResult};
use crate::entities::badge::Badge;
use crate::entities::badge_requirement::BadgeRequirement;
use crate::entities::progression_event_type::ProgressionEventType;
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
        badge_groups: Vec<(i32, i32)>,
        badges_earned: Vec<BetaApplicantBadgeDb>,
    ) -> AppResult<Vec<Badge>> {
        let group_map: HashMap<i32, i32> = badge_groups.into_iter().collect();
        let earned_map: HashMap<i32, BetaApplicantBadgeDb> = badges_earned
            .into_iter()
            .map(|badge| (badge.badge_id, badge))
            .collect();

        let result = badges
            .into_iter()
            .map(|badge| Badge {
                id: badge.id,
                title: badge.title,
                description: badge.description,
                score: badge.score,
                is_unlocked: earned_map.get(&badge.id).is_some(),
                unlocked_at: earned_map.get(&badge.id).map(|b| b.created_at),
                created_at: badge.created_at,
                badge_group_id: *group_map.get(&badge.id).unwrap_or(&0),
            })
            .collect();

        Ok(result)
    }
}

#[async_trait]
impl BadgePersistence for PostgresPersistence {
    async fn read_badges(&self, public_key: &str) -> AppResult<Vec<Badge>> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;

        let badges = sqlx::query_as!(
            BadgeDb,
            "SELECT b.* FROM badges b 
             INNER JOIN badge_group_conjunctions bgc ON b.id = bgc.badge_id 
             ORDER BY bgc.badge_group_id, bgc.sort_order"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        let badge_groups =
            sqlx::query!("SELECT badge_id, badge_group_id FROM badge_group_conjunctions")
                .fetch_all(&self.pool)
                .await
                .map_err(AppError::from)?
                .into_iter()
                .map(|row| (row.badge_id, row.badge_group_id))
                .collect::<Vec<(i32, i32)>>();

        let badges_earned = sqlx::query_as!(
            BetaApplicantBadgeDb,
            "SELECT badge_id, created_at FROM beta_applicant_badges WHERE beta_applicant_id = $1",
            applicant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(self.convert_to_badges(badges, badge_groups, badges_earned)?)
    }

    async fn read_badge_requirements(&self) -> AppResult<Vec<BadgeRequirement>> {
        let requirements = sqlx::query!(
            "SELECT bc.badge_id, pet.event_type, bc.operation, bc.required_count
             FROM badge_conditions bc
             INNER JOIN progression_event_types pet ON bc.progression_event_type_id = pet.id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        let result = requirements
            .into_iter()
            .map(|row| BadgeRequirement {
                badge_id: row.badge_id,
                progression_event_type: row.event_type,
                operation: row.operation,
                required_count: row.required_count,
            })
            .collect();

        Ok(result)
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

    async fn award_badge_if_eligible(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
        progress_count: i32,
    ) -> AppResult<()> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;

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
            event_type.id(),
            progress_count
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }
}
