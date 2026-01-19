use crate::app_error::{AppError, AppResult};
use crate::application::use_cases::mailchimp::MailchimpClient;
use crate::domain::entities::mailchimp::{
    MailchimpErrorResponse, MailchimpMemberRequest, MailchimpMemberResponse, email_to_hash,
};
use crate::infrastructure::mailchimp::MailchimpConfig;
use async_trait::async_trait;
use reqwest::Client;
use std::fmt::Debug;

#[derive(Debug)]
pub struct HttpMailchimpClient {
    client: Client,
    config: MailchimpConfig,
}

impl HttpMailchimpClient {
    pub fn new(config: MailchimpConfig) -> Self {
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json"),
                );
                headers
            })
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    fn get_member_url(&self, email: &str) -> String {
        let email_hash = email_to_hash(email);
        format!(
            "{}/lists/{}/members/{}",
            self.config.base_url(),
            self.config.list_id,
            email_hash
        )
    }
}

#[async_trait]
impl MailchimpClient for HttpMailchimpClient {
    async fn upsert_member(
        &self,
        email: &str,
        id: i32,
        public_key: &str,
        referral_code: &str,
    ) -> AppResult<()> {
        let request_body = MailchimpMemberRequest::new(email, id, public_key, referral_code);
        let url = self.get_member_url(email);

        let response = self
            .client
            .put(&url)
            .basic_auth("anystring", Some(&self.config.api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Mailchimp(format!("Network error: {}", e)))?;

        if response.status().is_success() {
            // Success - parse response to validate
            let _member_response: MailchimpMemberResponse = response
                .json::<MailchimpMemberResponse>()
                .await
                .map_err(|e| {
                    AppError::Internal(format!("Failed to parse Mailchimp response: {}", e))
                })?;
            Ok(())
        } else {
            // Error - try to parse error response for better error message
            let status = response.status();
            match response.json::<MailchimpErrorResponse>().await {
                Ok(error_response) => Err(AppError::Mailchimp(error_response.detail)),
                Err(_) => Err(AppError::Mailchimp(format!("API error: HTTP {}", status))),
            }
        }
    }

    async fn delete_member(&self, email: &str) -> AppResult<()> {
        let url = self.get_member_url(email);

        let response = self
            .client
            .delete(&url)
            .basic_auth("anystring", Some(&self.config.api_key))
            .send()
            .await
            .map_err(|e| AppError::Mailchimp(format!("Network error: {}", e)))?;

        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            // Success or member already doesn't exist
            Ok(())
        } else {
            // Error - try to parse error response
            let status = response.status();
            match response.json::<MailchimpErrorResponse>().await {
                Ok(error_response) => Err(AppError::Mailchimp(error_response.detail)),
                Err(_) => Err(AppError::Mailchimp(format!("API error: HTTP {}", status))),
            }
        }
    }
}
