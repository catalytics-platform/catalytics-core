use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::beta_applicant::BetaApplicant;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

// User struct as stored in the db.
#[derive(sqlx::FromRow, Debug)]
pub struct BetaApplicantDb {
    pub id: i32,
    pub public_key: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
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
    async fn create_beta_applicant(&self, public_key: &str) -> AppResult<BetaApplicant> {
        match sqlx::query_as!(
            BetaApplicantDb,
            "INSERT INTO beta_applicants (public_key) VALUES ($1) RETURNING id, public_key, email, created_at",
            public_key,
        )
        .fetch_one(&self.pool)
        .await {
            Ok(beta_applicant_db) => Ok(beta_applicant_db.into()),
            Err(sqlx::Error::Database(db_err)) if db_err.constraint().is_some() => {
                self.read_beta_applicant(public_key).await
            }
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn read_beta_applicant(&self, public_key: &str) -> AppResult<BetaApplicant> {
        let beta_applicant = sqlx::query_as!(
            BetaApplicantDb,
            "SELECT * FROM beta_applicants WHERE public_key = $1",
            public_key
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(beta_applicant.into())
    }

    async fn update_beta_applicant(
        &self,
        public_key: &str,
        email: &str,
    ) -> AppResult<BetaApplicant> {
        let beta_applicant_db = sqlx::query_as!(
            BetaApplicantDb,
            "UPDATE beta_applicants SET email = $1 WHERE public_key = $2 RETURNING id, public_key, email, created_at",
            email,
            public_key,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(beta_applicant_db.into())
    }
}
