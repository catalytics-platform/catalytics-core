use crate::adapters::http::app_state::AppState;
use crate::adapters::http::middleware::auth::AuthenticatedUser;
use crate::adapters::http::middleware::auth_middleware;
use crate::app_error::AppResult;
use crate::entities::beta_applicant::BetaApplicant;
use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use crate::use_cases::beta_applicant_progression::BetaApplicantProgressionUseCases;
use crate::use_cases::leaderboard::LeaderboardUseCases;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::{Json, Router, middleware};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

pub fn private_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_beta_applicant))
        .route("/", get(read_beta_applicant))
        .route("/", patch(update_beta_applicant))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn public_router() -> Router<AppState> {
    Router::new().route("/count", get(count_beta_applicant))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BetaApplicantResponse {
    public_key: String,
    email: Option<String>,
    registered_since: DateTime<Utc>,
    referral_code: String,
    referred_by: Option<String>,
    referral_count: i64,
    current_rank: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BetaApplicantsCountResponse {
    count: i64,
}

impl BetaApplicantResponse {
    fn from_applicant_with_rank(applicant: BetaApplicant, rank: u32) -> Self {
        Self {
            public_key: applicant.public_key,
            email: applicant.email,
            registered_since: applicant.created_at,
            referral_code: applicant.referral_code,
            referred_by: applicant.referred_by,
            referral_count: applicant.referral_count,
            current_rank: rank,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateBetaApplicantRequest {
    referral_code: Option<String>,
}

#[instrument(skip(beta_applicant_use_cases))]
async fn create_beta_applicant(
    auth: AuthenticatedUser,
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
    State(progression_use_cases): State<Arc<BetaApplicantProgressionUseCases>>,
    State(badge_use_cases): State<Arc<BadgeUseCases>>,
    State(leaderboard_use_cases): State<Arc<LeaderboardUseCases>>,
    Json(payload): Json<CreateBetaApplicantRequest>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases
        .create(
            &auth.public_key,
            payload.referral_code.as_deref(),
            progression_use_cases,
            badge_use_cases,
        )
        .await?;

    let rank = leaderboard_use_cases
        .get_user_rank(&auth.public_key)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(BetaApplicantResponse::from_applicant_with_rank(
            applicant, rank,
        )),
    ))
}

#[instrument(skip(beta_applicant_use_cases))]
async fn read_beta_applicant(
    auth: AuthenticatedUser,
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
    State(leaderboard_use_cases): State<Arc<LeaderboardUseCases>>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases.read(&auth.public_key).await?;
    let rank = leaderboard_use_cases
        .get_user_rank(&auth.public_key)
        .await?;

    Ok((
        StatusCode::OK,
        Json(BetaApplicantResponse::from_applicant_with_rank(
            applicant, rank,
        )),
    ))
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
    State(leaderboard_use_cases): State<Arc<LeaderboardUseCases>>,
    Json(payload): Json<UpdateBetaApplicantRequest>,
) -> AppResult<impl IntoResponse> {
    let applicant = beta_applicant_use_cases
        .update(&auth.public_key, &payload.email)
        .await?;
    let rank = leaderboard_use_cases
        .get_user_rank(&auth.public_key)
        .await?;

    Ok((
        StatusCode::OK,
        Json(BetaApplicantResponse::from_applicant_with_rank(
            applicant, rank,
        )),
    ))
}

#[instrument(skip(beta_applicant_use_cases))]
async fn count_beta_applicant(
    State(beta_applicant_use_cases): State<Arc<BetaApplicantUseCases>>,
) -> AppResult<impl IntoResponse> {
    let applicants_count = beta_applicant_use_cases.count().await?;

    Ok((
        StatusCode::OK,
        Json(BetaApplicantsCountResponse {
            count: applicants_count,
        }),
    ))
}
