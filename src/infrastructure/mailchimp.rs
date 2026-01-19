use std::env;

#[derive(Debug, Clone)]
pub struct MailchimpConfig {
    pub api_key: String,
    pub list_id: String,
    pub server_prefix: String,
}

impl Default for MailchimpConfig {
    fn default() -> Self {
        Self {
            api_key: env::var("MAILCHIMP_API_KEY").expect("MAILCHIMP_API_KEY must be set"),
            list_id: env::var("MAILCHIMP_LIST_ID").expect("MAILCHIMP_LIST_ID must be set"),
            server_prefix: env::var("MAILCHIMP_SERVER_PREFIX")
                .expect("MAILCHIMP_SERVER_PREFIX must be set"),
        }
    }
}

impl MailchimpConfig {
    pub fn base_url(&self) -> String {
        format!("https://{}.api.mailchimp.com/3.0", self.server_prefix)
    }
}
