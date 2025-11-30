use crate::adapters::http::app_state::AppState;
use crate::adapters::http::middleware::auth::AuthenticatedUser;
use crate::app_error::AppResult;
use crate::entities::beta_applicant::BetaApplicant;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::{middleware, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;
use crate::adapters::http::middleware::auth_middleware;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_beta_applicant))
        .route("/", get(read_beta_applicant))
        .route("/", patch(update_beta_applicant))
        .layer(middleware::from_fn(auth_middleware))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BetaApplicantResponse {
    public_key: String,
    email: Option<String>,
    registered_since: DateTime<Utc>,
}

impl From<BetaApplicant> for BetaApplicantResponse {
    fn from(applicant: BetaApplicant) -> Self {
        Self {
            public_key: applicant.public_key,
            email: applicant.email,
            registered_since: applicant.created_at,
        }
    }
}

#[instrument(skip(beta_applicant_use_cases))]
async fn create_beta_applicant(
    auth: AuthenticatedUser,
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases.create(&auth.public_key).await?;

    Ok((
        StatusCode::CREATED,
        Json(BetaApplicantResponse::from(applicant)),
    ))
}

#[instrument(skip(beta_applicant_use_cases))]
async fn read_beta_applicant(
    auth: AuthenticatedUser,
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases.read(&auth.public_key).await?;

    Ok((StatusCode::OK, Json(BetaApplicantResponse::from(applicant))))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateBetaApplicantRequest {
    email: String,
}

#[instrument(skip(beta_applicant_use_cases))]
async fn update_beta_applicant(
    auth: AuthenticatedUser,
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
    Json(payload): Json<UpdateBetaApplicantRequest>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases
        .update(&auth.public_key, &payload.email)
        .await?;

    Ok((StatusCode::OK, Json(BetaApplicantResponse::from(applicant))))
}
