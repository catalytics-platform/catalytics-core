use crate::adapters::client::mailchimp::HttpMailchimpClient;
use crate::adapters::persistence::PostgresPersistence;
use crate::infrastructure::database::init_db;
use crate::infrastructure::mailchimp::MailchimpConfig;
use crate::infrastructure::wallet_holdings::HttpWalletHoldingClient;

pub mod app;
pub mod database;
pub mod mailchimp;
pub mod setup;
pub mod wallet_holdings;

pub async fn postgres_persistence() -> anyhow::Result<PostgresPersistence> {
    let pool = init_db().await?;
    let persistence = PostgresPersistence::new(pool);
    Ok(persistence)
}

pub async fn wallet_holding_client() -> anyhow::Result<HttpWalletHoldingClient> {
    let client = HttpWalletHoldingClient::with_defaults()?;
    Ok(client)
}

pub async fn mailchimp_client() -> anyhow::Result<HttpMailchimpClient> {
    let config = MailchimpConfig::default();
    let client = HttpMailchimpClient::new(config);
    Ok(client)
}
