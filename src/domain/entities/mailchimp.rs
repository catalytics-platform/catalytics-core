use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct MailchimpMemberRequest {
    pub email_address: String,
    pub status: String,
    pub merge_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MailchimpMemberResponse {
    pub id: String,
    pub email_address: String,
    pub status: String,
    pub merge_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MailchimpErrorResponse {
    #[serde(rename = "type")]
    pub error_type: String,
    pub title: String,
    pub detail: String,
    pub status: u16,
}

impl MailchimpMemberRequest {
    pub fn new(email: &str, id: i32, public_key: &str, referral_code: &str) -> Self {
        let mut merge_fields = HashMap::new();
        // MMERGE1 = Id (number, required)
        merge_fields.insert(
            "MMERGE1".to_string(),
            serde_json::Value::Number(serde_json::Number::from(id)),
        );
        // MMERGE2 = PublicKey (text, required) - use truncated version
        merge_fields.insert(
            "MMERGE2".to_string(),
            serde_json::Value::String(truncate_public_key(public_key)),
        );
        // MMERGE3 = ReferralCode (text, required)
        merge_fields.insert(
            "MMERGE3".to_string(),
            serde_json::Value::String(referral_code.to_string()),
        );

        Self {
            email_address: email.to_string(),
            status: "subscribed".to_string(),
            merge_fields,
        }
    }
}

pub fn truncate_public_key(public_key: &str) -> String {
    if public_key.len() <= 8 {
        public_key.to_string()
    } else {
        format!(
            "{}...{}",
            &public_key[..4],
            &public_key[public_key.len() - 4..]
        )
    }
}

pub fn email_to_hash(email: &str) -> String {
    format!("{:x}", md5::compute(email.to_lowercase().as_bytes()))
}
