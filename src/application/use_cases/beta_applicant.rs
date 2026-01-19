use crate::app_error::AppResult;
use crate::entities::beta_applicant::BetaApplicant;
use crate::entities::progression_event_type::ProgressionEventType;
use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::beta_applicant_progression::BetaApplicantProgressionUseCases;
use crate::use_cases::leaderboard::LeaderboardUseCases;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

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
        email: Option<&str>,
    ) -> AppResult<BetaApplicant>;
    async fn count_beta_applicants(&self) -> AppResult<i64>;
    async fn count_referrals(&self, id: i32) -> AppResult<i64>;
    async fn count_referrals_by_public_key(&self, public_key: &str) -> AppResult<i32>;
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
        progression_use_cases: Arc<BetaApplicantProgressionUseCases>,
        badge_use_cases: Arc<BadgeUseCases>,
        leaderboard_use_cases: Arc<LeaderboardUseCases>,
    ) -> AppResult<BetaApplicant> {
        let applicant = self
            .persistence
            .create_beta_applicant(public_key, referral_code)
            .await?;

        // Add new user to leaderboard immediately after creation
        leaderboard_use_cases
            .add_new_user_to_leaderboard(applicant.id, public_key)
            .await?;

        progression_use_cases
            .record_progression_event(public_key, ProgressionEventType::BetaApplicantCreated, 1)
            .await?;

        badge_use_cases
            .award_badge_if_eligible(public_key, ProgressionEventType::BetaApplicantCreated, 1)
            .await?;

        progression_use_cases
            .sync_all_progressions(public_key, badge_use_cases)
            .await?;

        Ok(applicant)
    }

    pub async fn read(&self, public_key: &str) -> AppResult<BetaApplicant> {
        let applicant = self
            .persistence
            .read_beta_applicant_by_public_key(public_key)
            .await?;
        Ok(applicant)
    }

    pub async fn update(&self, public_key: &str, email: Option<&str>) -> AppResult<BetaApplicant> {
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
