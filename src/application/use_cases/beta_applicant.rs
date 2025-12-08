use std::fmt::Debug;
use crate::app_error::AppResult;
use crate::entities::beta_applicant::BetaApplicant;
use async_trait::async_trait;
use std::sync::Arc;
use crate::use_cases::badge::BadgeUseCases;

#[async_trait]
pub trait BetaApplicantPersistence: Send + Sync + Debug {
    async fn create_beta_applicant(
        &self,
        public_key: &str,
        referral_code: Option<&str>,
    ) -> AppResult<BetaApplicant>;
    async fn read_beta_applicant_by_id(&self, id: i32) -> AppResult<BetaApplicant>;
    async fn read_beta_applicant_by_public_key(&self, public_key: &str)
    -> AppResult<BetaApplicant>;
    async fn read_beta_applicant_by_referral_code(
        &self,
        referral_code: &str,
    ) -> AppResult<BetaApplicant>;
    async fn update_beta_applicant(
        &self,
        public_key: &str,
        email: &str,
    ) -> AppResult<BetaApplicant>;
    async fn count_beta_applicants(&self) -> AppResult<i64>;
    async fn count_referrals(&self, id: i32) -> AppResult<i64>;
}

#[derive(Clone, Debug)]
pub struct BetaApplicantUseCases {
    persistence: Arc<dyn BetaApplicantPersistence>,
}

impl BetaApplicantUseCases {
    pub fn new(persistence: Arc<dyn BetaApplicantPersistence>) -> Self {
        Self { persistence }
    }

    pub async fn create(
        &self,
        public_key: &str,
        referral_code: Option<&str>,
        badge_use_cases: Arc<BadgeUseCases>,
    ) -> AppResult<BetaApplicant> {
        let applicant = self
            .persistence
            .create_beta_applicant(public_key, referral_code)
            .await?;
        badge_use_cases.create_badge(public_key, 1, 1).await?;
        badge_use_cases.create_catics_badges(public_key).await?;
        Ok(applicant)
    }

    pub async fn read(&self, public_key: &str) -> AppResult<BetaApplicant> {
        let applicant = self
            .persistence
            .read_beta_applicant_by_public_key(public_key)
            .await?;
        Ok(applicant)
    }

    pub async fn update(&self, public_key: &str, email: &str) -> AppResult<BetaApplicant> {
        let applicant = self
            .persistence
            .update_beta_applicant(public_key, email)
            .await?;
        Ok(applicant)
    }

    pub async fn count(&self) -> AppResult<i64> {
        let applicant = self.persistence.count_beta_applicants().await?;
        Ok(applicant)
    }
}
