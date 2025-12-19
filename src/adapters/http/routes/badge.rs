use crate::adapters::http::app_state::AppState;
use crate::adapters::http::middleware::auth::AuthenticatedUser;
use crate::adapters::http::middleware::auth_middleware;
use crate::app_error::AppResult;
use crate::entities::badge::Badge;
use crate::use_cases::badge::BadgeUseCases;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router, middleware};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;
use crate::entities::badge_group::BadgeGroup;
use crate::use_cases::badge_group::BadgeGroupUseCases;

pub fn private_router() -> Router<AppState> {
    Router::new()
        .route("/", get(read_badges))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/sync", get(sync_user_badges))
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
}

impl From<Badge> for BadgeResponse {
    fn from(badge: Badge) -> Self {
        Self {
            id: badge.id,
            title: badge.title,
            description: badge.description,
            score: badge.score,
            is_unlocked: badge.is_unlocked,
            unlocked_at: badge.unlocked_at,
        }
    }
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
struct GetBadgesResponse {
    badges: Vec<BadgeResponse>,
    badge_groups: Vec<BadgeGroupResponse>,
}

#[instrument(skip(badge_use_cases))]
async fn read_badges(
    auth: AuthenticatedUser,
    State(badge_use_cases): State<Arc<BadgeUseCases>>,
    State(badge_group_use_cases): State<Arc<BadgeGroupUseCases>>,
) -> AppResult<impl IntoResponse> {
    let badges = badge_use_cases
        .read_all(&auth.public_key)
        .await?;
    let badge_groups = badge_group_use_cases.read_all().await?;

    let badges_response = badges.into_iter().map(BadgeResponse::from).collect::<Vec<_>>();
    let badge_groups_response = badge_groups.into_iter().map(BadgeGroupResponse::from).collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(
            GetBadgesResponse {
                badges: badges_response,
                badge_groups: badge_groups_response,
            }
        ),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncUserBadgesQueryParams {
    public_key: String,
}

#[instrument(skip(badge_use_cases))]
async fn sync_user_badges(
    Query(params): Query<SyncUserBadgesQueryParams>,
    State(badge_use_cases): State<Arc<BadgeUseCases>>,
) -> AppResult<impl IntoResponse> {
    badge_use_cases.sync_user_badges(&params.public_key).await?;
    Ok(StatusCode::OK)
}
