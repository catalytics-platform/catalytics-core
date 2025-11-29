use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{Engine as _, engine::general_purpose};
use bs58;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use tracing::{error, instrument, warn};
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

#[instrument(skip(request, next))]
pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = request.headers();

    let auth_header = extract_header(headers, "Authorization")?;
    let message = extract_header(headers, "X-Solana-Message")?;

    let auth_data = match parse_auth_headers(&auth_header, &message) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to parse auth headers: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if !is_authorized(&auth_data) {
        warn!("Authorization failed for request");
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

fn extract_header(headers: &HeaderMap, header_name: &str) -> Result<String, StatusCode> {
    headers
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            warn!("Missing or invalid header: {}", header_name);
            StatusCode::BAD_REQUEST
        })
}

fn parse_auth_headers(auth_header: &str, message: &str) -> Result<AuthData, String> {
    let solana = parse_solana_auth(auth_header)?;

    let mut decoded_message = urlencoding::decode(message)
        .map_err(|e| format!("Failed to URL decode message: {}", e))?
        .to_string();

    decoded_message = decoded_message.replace("\\n", "\n");

    Ok(AuthData {
        solana,
        decoded_message,
    })
}

fn parse_solana_auth(auth_header: &str) -> Result<SolanaAuth, String> {
    let auth_data = auth_header
        .strip_prefix("Solana ")
        .ok_or("Authorization header must start with 'Solana '")?;

    let parts: Vec<&str> = auth_data.split(':').collect();
    if parts.len() != 2 {
        return Err("Authorization data must be in format 'public_key:signature'".to_string());
    }

    let public_key = parts[0].to_string();
    let signature = parts[1].to_string();

    Ok(SolanaAuth {
        public_key,
        signature,
    })
}

fn is_authorized(auth_data: &AuthData) -> bool {
    verify_solana_signature(
        &auth_data.solana.public_key,
        &auth_data.solana.signature,
        &auth_data.decoded_message,
    )
}

fn verify_solana_signature(public_key_b58: &str, signature_base64: &str, message: &str) -> bool {
    let public_key_bytes = match bs58::decode(public_key_b58).into_vec() {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode public key from base58: {}", e);
            return false;
        }
    };

    let signature_bytes = match general_purpose::STANDARD.decode(signature_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode signature from base64: {}", e);
            return false;
        }
    };

    let verifying_key = match VerifyingKey::from_bytes(
        &public_key_bytes
            .try_into()
            .expect("Public key should be 32 bytes"),
    ) {
        Ok(key) => key,
        Err(e) => {
            error!("Invalid public key format: {}", e);
            return false;
        }
    };

    let signature = Signature::from_bytes(
        &signature_bytes
            .try_into()
            .expect("Signature should be 64 bytes"),
    );

    verifying_key.verify(message.as_bytes(), &signature).is_ok()
}
