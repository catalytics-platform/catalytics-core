use anyhow::{Context, Result};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{Engine as _, engine::general_purpose};
use bs58;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use urlencoding;

#[derive(Debug, Clone)]
struct SolanaAuth {
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Clone)]
struct AuthData {
    pub solana: SolanaAuth,
    pub decoded_message: String,
}

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = request.headers();

    let auth_header = extract_header(headers, "Authorization")?;
    let message = extract_header(headers, "X-Solana-Message")?;

    let auth_data =
        parse_auth_headers(&auth_header, &message).map_err(|_| StatusCode::BAD_REQUEST)?;

    is_authorized(&auth_data).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}

fn extract_header(headers: &HeaderMap, header_name: &str) -> Result<String, StatusCode> {
    headers
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(StatusCode::BAD_REQUEST)
}

fn parse_auth_headers(auth_header: &str, message: &str) -> Result<AuthData> {
    let solana = parse_solana_auth(auth_header)?;

    let mut decoded_message = urlencoding::decode(message)
        .context("Failed to URL decode message")?
        .to_string();

    decoded_message = decoded_message.replace("\\n", "\n");

    Ok(AuthData {
        solana,
        decoded_message,
    })
}

fn parse_solana_auth(auth_header: &str) -> Result<SolanaAuth> {
    let auth_data = auth_header
        .strip_prefix("Solana ")
        .context("Authorization header must start with 'Solana '")?;

    let parts: Vec<&str> = auth_data.split(':').collect();
    anyhow::ensure!(
        parts.len() == 2,
        "Authorization data must be in format 'public_key:signature'"
    );

    Ok(SolanaAuth {
        public_key: parts[0].to_string(),
        signature: parts[1].to_string(),
    })
}

fn is_authorized(auth_data: &AuthData) -> Result<()> {
    verify_solana_signature(
        &auth_data.solana.public_key,
        &auth_data.solana.signature,
        &auth_data.decoded_message,
    )
}

fn verify_solana_signature(
    public_key_b58: &str,
    signature_base64: &str,
    message: &str,
) -> Result<()> {
    let public_key_bytes = bs58::decode(public_key_b58)
        .into_vec()
        .context("Failed to decode public key from base58")?;

    let signature_bytes = general_purpose::STANDARD
        .decode(signature_base64)
        .context("Failed to decode signature from base64")?;

    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())
        .context("Invalid public key format")?;

    let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());

    verifying_key
        .verify(message.as_bytes(), &signature)
        .context("Signature verification failed")?;

    Ok(())
}
