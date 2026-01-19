use crate::app_error::AppResult;
use crate::entities::beta_applicant::BetaApplicant;
use crate::entities::progression_event_type::ProgressionEventType;
use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::beta_applicant_progression::BetaApplicantProgressionUseCases;
use crate::use_cases::leaderboard::LeaderboardUseCases;
use crate::use_cases::mailchimp::MailchimpClient;
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
    mailchimp_client: Arc<dyn MailchimpClient>,
}

impl BetaApplicantUseCases {
    pub fn new(
        persistence: Arc<dyn BetaApplicantPersistence>,
        mailchimp_client: Arc<dyn MailchimpClient>,
    ) -> Self {
        Self {
            persistence,
            mailchimp_client,
        }
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
        // 1. Get current applicant state BEFORE any updates
        let current_applicant = self
            .persistence
            .read_beta_applicant_by_public_key(public_key)
            .await?;

        // 2. Sync with Mailchimp FIRST - if this fails, nothing else happens
        match (current_applicant.email.as_deref(), email) {
            (None, Some(new_email)) => {
                // Create new subscriber
                self.mailchimp_client
                    .upsert_member(
                        new_email,
                        current_applicant.id,
                        public_key,
                        &current_applicant.referral_code,
                    )
                    .await?;
            }
            (Some(old_email), Some(new_email)) if old_email != new_email => {
                // First validate new email by trying to create it
                self.mailchimp_client
                    .upsert_member(
                        new_email,
                        current_applicant.id,
                        public_key,
                        &current_applicant.referral_code,
                    )
                    .await?;
                // Only delete old email entry if new email creation succeeded
                self.mailchimp_client.delete_member(old_email).await?;
            }
            (Some(old_email), None) => {
                // Delete subscriber
                self.mailchimp_client.delete_member(old_email).await?;
            }
            _ => {} // No sync needed (None->None, or same email)
        }

        // 3. Only update database AFTER Mailchimp succeeds
        let updated_applicant = self
            .persistence
            .update_beta_applicant(public_key, email)
            .await?;

        Ok(updated_applicant)
    }

    pub async fn count(&self) -> AppResult<i64> {
        let applicant = self.persistence.count_beta_applicants().await?;
        Ok(applicant)
    }
}
