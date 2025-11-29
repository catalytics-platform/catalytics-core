use async_trait::async_trait;
use chrono::NaiveDateTime;
use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::beta_applicant::BetaApplicant;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;

// User struct as stored in the db.
#[derive(sqlx::FromRow, Debug)]
pub struct BetaApplicantDb {
    pub id: u32,
    pub public_key: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

impl From<BetaApplicantDb> for BetaApplicant {
    fn from(user_db: BetaApplicantDb) -> Self {
        Self {
            id: user_db.id,
            public_key: user_db.public_key,
            email: user_db.email,
            created_at: user_db.created_at,
        }
    }
}

#[async_trait]
impl BetaApplicantPersistence for PostgresPersistence {
    async fn create_beta_applicant(&self, public_key: &str, email: &str) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO beta_applicants (public_key, email) VALUES ($1, $2)",
            public_key, email,
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }
}