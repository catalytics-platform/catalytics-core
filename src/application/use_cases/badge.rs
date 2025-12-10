use crate::app_error::AppResult;
use crate::entities::badge::Badge;
use crate::use_cases::wallet_holdings::WalletHoldingsClient;
use async_trait::async_trait;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait BadgePersistence: Send + Sync + Debug {
    async fn read_badges(&self, public_key: &str, group_id: i32) -> AppResult<Vec<Badge>>;
    async fn create_badge(&self, public_key: &str, badge_id: i32, value: i32) -> AppResult<()>;
    async fn create_catics_badges(&self, public_key: &str, catics_balance: f64) -> AppResult<()>;
    async fn create_staked_jup_badges(
        &self,
        public_key: &str,
        catics_balance: f64,
    ) -> AppResult<()>;
    async fn create_badges_for_progression(
        &self,
        applicant_id: i32,
        progression_event_type_id: i32,
        progress_count: i32,
    ) -> AppResult<()>;
}

#[derive(Clone, Debug)]
pub struct BadgeUseCases {
    persistence: Arc<dyn BadgePersistence>,
    wallet_holdings_client: Arc<dyn WalletHoldingsClient>,
}

impl BadgeUseCases {
    pub fn new(
        persistence: Arc<dyn BadgePersistence>,
        wallet_holdings_client: Arc<dyn WalletHoldingsClient>,
    ) -> Self {
        Self {
            persistence,
            wallet_holdings_client,
        }
    }

    pub async fn read_all(&self, public_key: &str, group_id: i32) -> AppResult<Vec<Badge>> {
        self.create_catics_badges(public_key).await?;
        self.create_staked_jup_badges(public_key).await?;
        let badges = self.persistence.read_badges(public_key, group_id).await?;
        Ok(badges)
    }

    pub async fn create_badge(&self, public_key: &str, badge_id: i32, value: i32) -> AppResult<()> {
        Ok(self
            .persistence
            .create_badge(public_key, badge_id, value)
            .await?)
    }

    pub async fn create_catics_badges(&self, public_key: &str) -> AppResult<()> {
        let catics_balance = self
            .wallet_holdings_client
            .get_token_balance(&public_key, &env::var("CATICS_TOKEN_ADDRESS").unwrap())
            .await?;
        Ok(self
            .persistence
            .create_catics_badges(public_key, catics_balance)
            .await?)
    }

    pub async fn create_staked_jup_badges(&self, public_key: &str) -> AppResult<()> {
        let staked_jup_balance = self
            .wallet_holdings_client
            .get_staked_token_balance(&public_key, &env::var("JUP_TOKEN_ADDRESS").unwrap())
            .await?;
        Ok(self
            .persistence
            .create_staked_jup_badges(public_key, staked_jup_balance)
            .await?)
    }
}
