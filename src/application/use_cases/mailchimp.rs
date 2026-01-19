use crate::app_error::AppResult;
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait MailchimpClient: Send + Sync + Debug {
    async fn upsert_member(
        &self,
        email: &str,
        id: i32,
        public_key: &str,
        referral_code: &str,
    ) -> AppResult<()>;
    async fn delete_member(&self, email: &str) -> AppResult<()>;
}
