use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use crate::app_error::AppResult;
use crate::entities::wallet_holdings::WalletHoldings;

#[async_trait]
pub trait WalletHoldingsClient: Send + Sync + Debug {
    async fn get_wallet_holdings(&self, address: &str) -> AppResult<WalletHoldings>;
    async fn get_token_balance(&self, address: &str, token_address: &str) -> AppResult<f64>;
}

#[derive(Clone, Debug)]
pub struct WalletHoldingsUseCases {
    client: Arc<dyn WalletHoldingsClient>
}

impl WalletHoldingsUseCases {
    pub fn new(client: Arc<dyn WalletHoldingsClient>) -> Self {
        Self { client }
    }

    pub async fn read_token_balance(&self, address: &str, token_address: &str) -> AppResult<f64> {
        let balance = self.client.get_token_balance(address, token_address).await?;
        Ok(balance)
    }
}