use crate::app_error::AppResult;
use crate::entities::beta_applicant::BetaApplicant;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait BetaApplicantPersistence: Send + Sync {
    async fn create_beta_applicant(&self, public_key: &str) -> AppResult<BetaApplicant>;
    async fn read_beta_applicant(&self, public_key: &str) -> AppResult<BetaApplicant>;
}

#[derive(Clone)]
pub struct BetaApplicantUseCases {
    persistence: Arc<dyn BetaApplicantPersistence>,
}

impl BetaApplicantUseCases {
    pub fn new(persistence: Arc<dyn BetaApplicantPersistence>) -> Self {
        Self { persistence }
    }

    pub async fn create(&self, public_key: &str) -> AppResult<BetaApplicant> {
        let applicant = self.persistence.create_beta_applicant(public_key).await?;
        Ok(applicant)
    }

    pub async fn read(&self, public_key: &str) -> AppResult<BetaApplicant> {
        let applicant = self.persistence.read_beta_applicant(public_key).await?;
        Ok(applicant)
    }
}
