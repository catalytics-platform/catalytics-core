use crate::app_error::AppResult;
use crate::entities::wallet_holdings::WalletHoldings;
use crate::use_cases::wallet_holdings::WalletHoldingsClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub(crate) use crate::infrastructure::jupiter::HttpJupiterClient;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JupiterHoldingsResponse {
    pub amount: String,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
    pub tokens: Option<HashMap<String, Vec<JupiterHoldingsTokenResponse>>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JupiterHoldingsTokenResponse {
    pub account: String,
    pub amount: String,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
    #[serde(rename = "isFrozen")]
    pub is_frozen: bool,
    #[serde(rename = "isAssociatedTokenAccount")]
    pub is_associated_token_account: bool,
    pub decimals: i32,
    #[serde(rename = "programId")]
    pub program_id: String,
    #[serde(rename = "excludeFromNetWorth")]
    pub exclude_from_net_worth: bool,
}

#[async_trait]
impl WalletHoldingsClient for HttpJupiterClient {
    async fn get_wallet_holdings(&self, public_key: &str) -> AppResult<WalletHoldings> {
        let endpoint = format!("/ultra/v1/holdings/{}", public_key);
        Ok(self.convert_to_wallet_holdings(public_key.to_string(), self.make_get_request(&endpoint).await?).await?)
    }

    async fn get_token_balance(&self, public_key: &str, token_address: &str) -> AppResult<f64> {
        let wallet_holdings = self.get_wallet_holdings(public_key).await?;
        match wallet_holdings.token_holdings.get(token_address) {
            Some(balance) => Ok(*balance),
            None => Ok(0.0),
        }
    }
}

