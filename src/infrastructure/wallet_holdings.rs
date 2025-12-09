use crate::adapters::client::wallet_holdings::{JupiterHoldingsResponse, StakedJupResponse};
use crate::app_error::{AppError, AppResult};
use crate::entities::wallet_holdings::WalletHoldings;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
pub struct HttpWalletHoldingsConfig {
    pub jupiter_base_url: String,
    pub jupiter_api_key: String,
    pub catalytics_base_url: String,
}

impl Default for HttpWalletHoldingsConfig {
    fn default() -> Self {
        Self {
            jupiter_base_url: env::var("JUPITER_API_BASE_URL").unwrap(),
            jupiter_api_key: env::var("JUPITER_API_KEY").unwrap(),
            catalytics_base_url: env::var("CATALYTICS_API_BASE_URL").unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct HttpWalletHoldingClient {
    pub jupiter_client: Client,
    pub catalytics_client: Client,
    pub config: HttpWalletHoldingsConfig,
}

impl HttpWalletHoldingClient {
    pub fn new(config: HttpWalletHoldingsConfig) -> AppResult<Self> {
        let mut headers = HeaderMap::new();

        let api_key_header = HeaderValue::from_str(&config.jupiter_api_key)
            .map_err(|e| AppError::Internal(format!("Invalid API key format: {}", e)))?;
        headers.insert("x-api-key", api_key_header);

        let jupiter_client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build client: {}", e)))?;
        let catalytics_client = Client::builder()
            .default_headers(HeaderMap::new())
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build client: {}", e)))?;

        Ok(Self {
            jupiter_client,
            catalytics_client,
            config,
        })
    }

    pub fn with_defaults() -> AppResult<Self> {
        Self::new(HttpWalletHoldingsConfig::default())
    }

    pub async fn make_jupiter_get_request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> AppResult<T> {
        let url = format!("{}{}", self.config.jupiter_base_url, endpoint);

        let response = self
            .jupiter_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP request failed: {}", e)))?;
        if !response.status().is_success() {
            return Err(AppError::Internal(format!(
                "Jupiter API request failed with status {}: {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string())
            )));
        }
        let body = response
            .json::<T>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse JSON response: {}", e)))?;
        Ok(body)
    }

    pub async fn make_catalytics_get_request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> AppResult<T> {
        let url = format!("{}{}", self.config.catalytics_base_url, endpoint);

        let response = self
            .catalytics_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP request failed: {}", e)))?;
        if !response.status().is_success() {
            return Err(AppError::Internal(format!(
                "Jupiter API request failed with status {}: {}",
                response.status(),
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string())
            )));
        }
        let body = response
            .json::<T>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse JSON response: {}", e)))?;
        Ok(body)
    }

    pub async fn convert_to_wallet_holdings(
        &self,
        public_key: String,
        holdings: JupiterHoldingsResponse,
        staked_jup: StakedJupResponse,
    ) -> AppResult<WalletHoldings> {
        let mut token_holdings = HashMap::new();

        if let Some(tokens) = holdings.tokens {
            for (token_mint, token_accounts) in tokens {
                let total_balance: f64 =
                    token_accounts.iter().map(|account| account.ui_amount).sum();

                if total_balance > 0.0 {
                    token_holdings.insert(token_mint, total_balance);
                }
            }
        }

        let mut staked_token_holdings = HashMap::new();
        staked_token_holdings.insert(
            env::var("JUP_TOKEN_ADDRESS").unwrap(),
            staked_jup.staked_jup,
        );

        Ok(WalletHoldings {
            public_key,
            token_holdings,
            staked_token_holdings,
        })
    }
}
