use std::collections::HashMap;
use std::env;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::Client;
use crate::adapters::client::jupiter::JupiterHoldingsResponse;
use crate::app_error::{AppError, AppResult};
use crate::entities::wallet_holdings::WalletHoldings;

#[derive(Debug, Clone)]
pub struct JupiterConfig {
    pub base_url: String,
    pub api_key: String,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            base_url: env::var("JUPITER_API_BASE_URL").unwrap(),
            api_key: env::var("JUPITER_API_KEY").unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct HttpJupiterClient {
    pub client: Client,
    pub config: JupiterConfig,
}

impl HttpJupiterClient {
    pub fn new(config: JupiterConfig) -> AppResult<Self> {
        let mut headers = HeaderMap::new();

        let api_key_header = HeaderValue::from_str(&config.api_key)
            .map_err(|e| AppError::Internal(format!("Invalid API key format: {}", e)))?;
        headers.insert("x-api-key", api_key_header);

        let client = Client::builder().default_headers(headers).build().map_err(|e| AppError::Internal(format!("Failed to build client: {}", e)))?;
        Ok(Self { client, config })
    }

    pub fn with_defaults() -> AppResult<Self> {
        Self::new(JupiterConfig::default())
    }

    pub async fn make_get_request<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> AppResult<T> {
        let url = format!("{}{}", self.config.base_url, endpoint);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP request failed: {}", e)))?;
        if !response.status().is_success() {
            return Err(AppError::Internal(format!(
                "Jupiter API request failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            )));
        }
        let body = response
            .json::<T>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse JSON response: {}", e)))?;
        Ok(body)
    }

    pub async fn convert_to_wallet_holdings(&self, public_key: String, holdings: JupiterHoldingsResponse) -> AppResult<WalletHoldings> {
        let mut token_holdings = HashMap::new();

        if let Some(tokens) = holdings.tokens {
            for (token_mint, token_accounts) in tokens {
                let total_balance: f64 = token_accounts.iter().map(|account| account.ui_amount).sum();

                if total_balance > 0.0 {
                    token_holdings.insert(token_mint, total_balance);
                }
            }
        }

        Ok(WalletHoldings {
            public_key,
            token_holdings
        })
    }
}