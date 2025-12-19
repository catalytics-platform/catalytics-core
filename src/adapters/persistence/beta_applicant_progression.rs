use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::progression_event_type::ProgressionEventType;
use crate::entities::user_progression::UserProgression;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;
use crate::use_cases::beta_applicant_progression::BetaApplicantProgressionPersistence;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
impl BetaApplicantProgressionPersistence for PostgresPersistence {
    async fn record_progression_event(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
        progress_count: i32,
    ) -> AppResult<()> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;

        sqlx::query!(
            r#"
            INSERT INTO beta_applicant_progressions (beta_applicant_id, progression_event_type_id, progress_count)
            VALUES ($1, $2, $3)
            ON CONFLICT (beta_applicant_id, progression_event_type_id)
            DO UPDATE SET progress_count = EXCLUDED.progress_count
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

    async fn read_user_progressions(&self, public_key: &str) -> AppResult<Vec<UserProgression>> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;

        let user_progress = sqlx::query!(
            "SELECT bap.progression_event_type_id, bap.progress_count, pet.event_type
             FROM beta_applicant_progressions bap
             INNER JOIN progression_event_types pet ON bap.progression_event_type_id = pet.id  
             WHERE bap.beta_applicant_id = $1",
            applicant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        let all_types =
            sqlx::query!("SELECT id, event_type FROM progression_event_types ORDER BY id")
                .fetch_all(&self.pool)
                .await
                .map_err(AppError::from)?;

        let user_map: HashMap<String, i32> = user_progress
            .into_iter()
            .map(|row| (row.event_type, row.progress_count))
            .collect();

        let result = all_types
            .into_iter()
            .map(|row| UserProgression {
                progression_event_type: row.event_type.clone(),
                current_progress: *user_map.get(&row.event_type).unwrap_or(&0),
            })
            .collect();

        Ok(result)
    }

    async fn get_user_progression(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
    ) -> AppResult<i32> {
        let applicant_id = self.read_beta_applicant_by_public_key(public_key).await?.id;

        let progress = sqlx::query_scalar!(
            "SELECT progress_count FROM beta_applicant_progressions 
             WHERE beta_applicant_id = $1 AND progression_event_type_id = $2",
            applicant_id,
            event_type.id()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(progress.unwrap_or(0))
    }
}
