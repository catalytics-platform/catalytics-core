use crate::adapters::http::app_state::AppState;
use crate::adapters::http::middleware::auth::AuthenticatedUser;
use crate::adapters::http::middleware::auth_middleware;
use crate::app_error::AppResult;
use crate::entities::badge::BadgeDto;
use crate::entities::badge_group::BadgeGroup;
use crate::entities::badge_requirement::BadgeRequirementDto;
use crate::entities::user_progression::UserProgressionDto;
use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::badge_group::BadgeGroupUseCases;
use crate::use_cases::beta_applicant_progression::BetaApplicantProgressionUseCases;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router, middleware};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;

pub fn private_router() -> Router<AppState> {
    Router::new()
        .route("/", get(read_badges))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn public_router() -> Router<AppState> {
    Router::new().route("/sync", get(sync_user_badges))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BadgeResponse {
    id: i32,
    title: String,
    description: String,
    score: i32,
    is_unlocked: bool,
    unlocked_at: Option<DateTime<Utc>>,
    badge_group_id: i32,
    progression_event_type: String,
    operation: String,
    required_count: i32,
}

fn assemble_badge_responses(
    badge_dtos: Vec<BadgeDto>,
    requirement_dtos: Vec<BadgeRequirementDto>,
) -> Vec<BadgeResponse> {
    let requirements_map: HashMap<i32, &BadgeRequirementDto> = requirement_dtos
        .iter()
        .map(|req| (req.badge_id, req))
        .collect();

    badge_dtos
        .into_iter()
        .map(|badge| {
            let requirement = requirements_map.get(&badge.id);
            BadgeResponse {
                id: badge.id,
                title: badge.title,
                description: badge.description,
                score: badge.score,
                is_unlocked: badge.is_unlocked,
                unlocked_at: badge.unlocked_at,
                badge_group_id: badge.badge_group_id,
                progression_event_type: requirement
                    .map(|r| r.progression_event_type.clone())
                    .unwrap_or_default(),
                operation: requirement.map(|r| r.operation.clone()).unwrap_or_default(),
                required_count: requirement.map(|r| r.required_count).unwrap_or(0),
            }
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BadgeGroupResponse {
    id: i32,
    title: String,
    description: String,
}

impl From<BadgeGroup> for BadgeGroupResponse {
    fn from(badge_group: BadgeGroup) -> Self {
        Self {
            id: badge_group.id,
            title: badge_group.title,
            description: badge_group.description,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserProgressionResponse {
    progression_event_type: String,
    current_progress: i32,
}

impl From<UserProgressionDto> for UserProgressionResponse {
    fn from(progression: UserProgressionDto) -> Self {
        Self {
            progression_event_type: progression.progression_event_type,
            current_progress: progression.current_progress,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetBadgesResponse {
    badges: Vec<BadgeResponse>,
    badge_groups: Vec<BadgeGroupResponse>,
    user_progressions: Vec<UserProgressionResponse>,
}

#[instrument(skip(badge_use_cases))]
async fn read_badges(
    auth: AuthenticatedUser,
    State(badge_use_cases): State<Arc<BadgeUseCases>>,
    State(badge_group_use_cases): State<Arc<BadgeGroupUseCases>>,
    State(progression_use_cases): State<Arc<BetaApplicantProgressionUseCases>>,
) -> AppResult<impl IntoResponse> {
    let badges = badge_use_cases.read_all(&auth.public_key).await?;
    let badge_requirements = badge_use_cases.read_badge_requirements().await?;
    let badge_groups = badge_group_use_cases.read_all().await?;
    let user_progressions = progression_use_cases
        .read_user_progressions(&auth.public_key)
        .await?;

    let badges_response = assemble_badge_responses(badges, badge_requirements);
    let badge_groups_response = badge_groups
        .into_iter()
        .map(BadgeGroupResponse::from)
        .collect::<Vec<_>>();
    let user_progressions_response = user_progressions
        .into_iter()
        .map(UserProgressionResponse::from)
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(GetBadgesResponse {
            badges: badges_response,
            badge_groups: badge_groups_response,
            user_progressions: user_progressions_response,
        }),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncUserBadgesQueryParams {
    public_key: String,
}

#[instrument(skip(progression_use_cases, badge_use_cases))]
async fn sync_user_badges(
    Query(params): Query<SyncUserBadgesQueryParams>,
    State(progression_use_cases): State<Arc<BetaApplicantProgressionUseCases>>,
    State(badge_use_cases): State<Arc<BadgeUseCases>>,
) -> AppResult<impl IntoResponse> {
    progression_use_cases
        .sync_all_progressions(&params.public_key, badge_use_cases)
        .await?;
    Ok(StatusCode::OK)
}
