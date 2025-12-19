use crate::app_error::AppResult;
use crate::entities::progression_event_type::ProgressionEventType;
use crate::entities::user_progression::{UserProgression, UserProgressionDto};
use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::beta_applicant::BetaApplicantPersistence;
use crate::use_cases::wallet_holdings::WalletHoldingsClient;
use async_trait::async_trait;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{error, info};

#[async_trait]
pub trait BetaApplicantProgressionPersistence: Send + Sync + Debug {
    async fn record_progression_event(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
        progress_count: i32,
    ) -> AppResult<()>;
    async fn read_user_progressions(&self, public_key: &str) -> AppResult<Vec<UserProgression>>;
    async fn get_user_progression(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
    ) -> AppResult<i32>;
}

#[derive(Clone, Debug)]
pub struct BetaApplicantProgressionUseCases {
    persistence: Arc<dyn BetaApplicantProgressionPersistence>,
    beta_applicant_persistence: Arc<dyn BetaApplicantPersistence>,
    wallet_holdings_client: Arc<dyn WalletHoldingsClient>,
}

impl BetaApplicantProgressionUseCases {
    pub fn new(
        persistence: Arc<dyn BetaApplicantProgressionPersistence>,
        beta_applicant_persistence: Arc<dyn BetaApplicantPersistence>,
        wallet_holdings_client: Arc<dyn WalletHoldingsClient>,
    ) -> Self {
        Self {
            persistence,
            beta_applicant_persistence,
            wallet_holdings_client,
        }
    }

    pub async fn record_progression_event(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
        progress_count: i32,
    ) -> AppResult<()> {
        self.persistence
            .record_progression_event(public_key, event_type, progress_count)
            .await
    }

    pub async fn read_user_progressions(
        &self,
        public_key: &str,
    ) -> AppResult<Vec<UserProgressionDto>> {
        let progressions = self.persistence.read_user_progressions(public_key).await?;
        Ok(progressions
            .into_iter()
            .map(UserProgressionDto::from)
            .collect())
    }

    pub async fn get_user_progression(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
    ) -> AppResult<i32> {
        self.persistence
            .get_user_progression(public_key, event_type)
            .await
    }

    pub async fn sync_all_progressions(
        &self,
        public_key: &str,
        badge_use_cases: Arc<BadgeUseCases>,
    ) -> AppResult<()> {
        info!("Starting progression sync for user: {}", public_key);

        let beta_applicant_progress = self
            .sync_beta_applicant_created_progression(public_key)
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Failed to sync beta applicant progression for {}: {}",
                    public_key, e
                );
                0
            });

        let catics_progress = self
            .sync_catics_balance_progression(public_key)
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Failed to sync CATICS balance progression for {}: {}",
                    public_key, e
                );
                0
            });

        let jup_progress = self
            .sync_jup_staked_progression(public_key)
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Failed to sync JUP staked progression for {}: {}",
                    public_key, e
                );
                0
            });

        let referral_progress = self
            .sync_referral_created_progression(public_key)
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Failed to sync referral progression for {}: {}",
                    public_key, e
                );
                0
            });

        if let Err(e) = badge_use_cases
            .award_badge_if_eligible(
                public_key,
                ProgressionEventType::BetaApplicantCreated,
                beta_applicant_progress,
            )
            .await
        {
            error!(
                "Failed to award beta applicant badge for {}: {}",
                public_key, e
            );
        }

        if let Err(e) = badge_use_cases
            .award_badge_if_eligible(
                public_key,
                ProgressionEventType::CaticsBalanceCheck,
                catics_progress,
            )
            .await
        {
            error!("Failed to award CATICS badge for {}: {}", public_key, e);
        }

        if let Err(e) = badge_use_cases
            .award_badge_if_eligible(public_key, ProgressionEventType::JupStaked, jup_progress)
            .await
        {
            error!("Failed to award JUP badge for {}: {}", public_key, e);
        }

        if let Err(e) = badge_use_cases
            .award_badge_if_eligible(
                public_key,
                ProgressionEventType::ReferralCreated,
                referral_progress,
            )
            .await
        {
            error!("Failed to award referral badge for {}: {}", public_key, e);
        }

        info!("Completed progression sync for user: {}", public_key);
        Ok(())
    }

    async fn sync_beta_applicant_created_progression(&self, public_key: &str) -> AppResult<i32> {
        self.record_progression_event(public_key, ProgressionEventType::BetaApplicantCreated, 1)
            .await?;
        info!(
            "Synced beta applicant created progression for {}: 1",
            public_key
        );
        Ok(1)
    }

    async fn sync_catics_balance_progression(&self, public_key: &str) -> AppResult<i32> {
        let catics_token_address = env::var("CATICS_TOKEN_ADDRESS").map_err(|_| {
            crate::app_error::AppError::Internal("CATICS_TOKEN_ADDRESS not set".to_string())
        })?;

        let balance = self
            .wallet_holdings_client
            .get_token_balance(public_key, &catics_token_address)
            .await?;
        let balance_int = balance as i32;

        self.record_progression_event(
            public_key,
            ProgressionEventType::CaticsBalanceCheck,
            balance_int,
        )
        .await?;

        info!(
            "Synced CATICS balance progression for {}: {}",
            public_key, balance_int
        );
        Ok(balance_int)
    }

    async fn sync_jup_staked_progression(&self, public_key: &str) -> AppResult<i32> {
        let jup_token_address = env::var("JUP_TOKEN_ADDRESS").map_err(|_| {
            crate::app_error::AppError::Internal("JUP_TOKEN_ADDRESS not set".to_string())
        })?;

        let staked = self
            .wallet_holdings_client
            .get_staked_token_balance(public_key, &jup_token_address)
            .await?;
        let staked_int = staked as i32;

        self.record_progression_event(public_key, ProgressionEventType::JupStaked, staked_int)
            .await?;

        info!(
            "Synced JUP staked progression for {}: {}",
            public_key, staked_int
        );
        Ok(staked_int)
    }

    async fn sync_referral_created_progression(&self, public_key: &str) -> AppResult<i32> {
        let count = self
            .beta_applicant_persistence
            .count_referrals_by_public_key(public_key)
            .await?;

        self.record_progression_event(public_key, ProgressionEventType::ReferralCreated, count)
            .await?;

        info!("Synced referral progression for {}: {}", public_key, count);
        Ok(count)
    }
}
