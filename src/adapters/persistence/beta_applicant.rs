use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::beta_applicant::BetaApplicant;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::Rng;
use rand::distr::Alphanumeric;

#[derive(sqlx::FromRow, Debug)]
pub struct BetaApplicantDb {
    pub id: i32,
    pub public_key: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub referral_code: String,
    pub referred_by_id: Option<i32>,
}

impl PostgresPersistence {
    async fn get_referrer_wallet(&self, referred_by_id: Option<i32>) -> Option<String> {
        match referred_by_id {
            Some(id) => match self.read_beta_applicant_by_id(id).await {
                Ok(referrer) => {
                    let public_key = &referrer.public_key;
                    if public_key.len() > 10 {
                        Some(format!(
                            "{}...{}",
                            &public_key[..4],
                            &public_key[public_key.len() - 4..]
                        ))
                    } else {
                        Some(public_key.clone())
                    }
                }
                Err(_) => None,
            },
            None => None,
        }
    }

    async fn convert_to_beta_applicant(&self, db: BetaApplicantDb) -> AppResult<BetaApplicant> {
        let referred_by = self.get_referrer_wallet(db.referred_by_id).await;

        Ok(BetaApplicant {
            id: db.id,
            public_key: db.public_key,
            email: db.email,
            created_at: db.created_at,
            referral_code: db.referral_code,
            referred_by,
        })
    }
}

#[async_trait]
impl BetaApplicantPersistence for PostgresPersistence {
    async fn create_beta_applicant(
        &self,
        public_key: &str,
        referral_code: Option<&str>,
    ) -> AppResult<BetaApplicant> {
        let new_referral_code: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        let referrer_id = match referral_code {
            Some(referral_code_value) => Some(
                self.read_beta_applicant_by_referral_code(referral_code_value)
                    .await?
                    .id,
            ),
            None => None,
        };

        match sqlx::query_as!(
            BetaApplicantDb,
            "INSERT INTO beta_applicants (public_key, referral_code, referred_by_id) VALUES ($1, $2, $3) RETURNING id, public_key, email, created_at, referral_code, referred_by_id",
            public_key,
            new_referral_code,
            referrer_id,
        )
        .fetch_one(&self.pool)
        .await {
            Ok(beta_applicant) => Ok(self.convert_to_beta_applicant(beta_applicant).await?),
            Err(sqlx::Error::Database(db_err)) if db_err.constraint().is_some() => {
                self.read_beta_applicant_by_public_key(public_key).await
            }
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn read_beta_applicant_by_id(&self, id: i32) -> AppResult<BetaApplicant> {
        let beta_applicant = sqlx::query_as!(
            BetaApplicantDb,
            "SELECT * FROM beta_applicants WHERE id = $1",
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(self.convert_to_beta_applicant(beta_applicant).await?)
    }

    async fn read_beta_applicant_by_public_key(
        &self,
        public_key: &str,
    ) -> AppResult<BetaApplicant> {
        let beta_applicant = sqlx::query_as!(
            BetaApplicantDb,
            "SELECT * FROM beta_applicants WHERE public_key = $1",
            public_key
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(self.convert_to_beta_applicant(beta_applicant).await?)
    }

    async fn read_beta_applicant_by_referral_code(
        &self,
        referral_code: &str,
    ) -> AppResult<BetaApplicant> {
        let beta_applicant = sqlx::query_as!(
            BetaApplicantDb,
            "SELECT * FROM beta_applicants WHERE referral_code = $1",
            referral_code
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(self.convert_to_beta_applicant(beta_applicant).await?)
    }

    async fn update_beta_applicant(
        &self,
        public_key: &str,
        email: &str,
    ) -> AppResult<BetaApplicant> {
        let beta_applicant = sqlx::query_as!(
            BetaApplicantDb,
            "UPDATE beta_applicants SET email = $1 WHERE public_key = $2 RETURNING id, public_key, email, created_at, referral_code, referred_by_id",
            email,
            public_key,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(self.convert_to_beta_applicant(beta_applicant).await?)
    }
}
