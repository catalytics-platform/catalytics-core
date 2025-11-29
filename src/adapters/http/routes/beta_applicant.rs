use crate::adapters::http::app_state::AppState;
use crate::app_error::AppResult;
use crate::entities::beta_applicant::BetaApplicant;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_beta_applicant))
        .route("/{user_id}", get(read_beta_applicant))
}

#[derive(Debug, Clone, Deserialize)]
struct CreateBetaApplicantRequest {
    public_key: String,
}

#[derive(Debug, Clone, Serialize)]
struct CreateBetaApplicantResponse {
    public_key: String,
    email: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<BetaApplicant> for CreateBetaApplicantResponse {
    fn from(applicant: BetaApplicant) -> Self {
        Self {
            public_key: applicant.public_key,
            email: applicant.email,
            created_at: applicant.created_at,
        }
    }
}

#[instrument(skip(beta_applicant_use_cases))]
async fn create_beta_applicant(
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
    Json(payload): Json<CreateBetaApplicantRequest>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases.create(&payload.public_key).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateBetaApplicantResponse::from(applicant)),
    ))
}

#[instrument(skip(beta_applicant_use_cases))]
async fn read_beta_applicant(
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
    Path(user_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases.read(&user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateBetaApplicantResponse::from(applicant)),
    ))
}
