use crate::app_error::AppResult;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait BetaApplicantPersistence: Send + Sync {
    async fn create_beta_applicant(&self, public_key: &str, email: &str) -> AppResult<()>;
}

#[derive(Clone)]
pub struct BetaApplicantUseCases {
    persistence: Arc<dyn BetaApplicantPersistence>,
}

impl BetaApplicantUseCases {
    pub fn new(persistence: Arc<dyn BetaApplicantPersistence>) -> Self {
        Self { persistence }
    }

    pub async fn add(&self, public_key: &str, email: &str) -> AppResult<()> {
        self.persistence
            .create_beta_applicant(public_key, email)
            .await?;
        Ok(())
    }
}
