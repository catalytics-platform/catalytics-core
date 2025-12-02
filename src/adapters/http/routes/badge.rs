use crate::adapters::http::app_state::AppState;
use crate::adapters::http::middleware::auth::AuthenticatedUser;
use crate::adapters::http::middleware::auth_middleware;
use crate::app_error::AppResult;
use crate::entities::badge::Badge;
use crate::use_cases::badge::BadgeUseCases;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router, middleware};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(read_badges))
        .layer(middleware::from_fn(auth_middleware))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BadgeResponse {
    id: i32,
    title: String,
    description: String,
    score: i32,
    is_active: bool,
    earned_at: Option<DateTime<Utc>>,
}

impl From<Badge> for BadgeResponse {
    fn from(badge: Badge) -> Self {
        Self {
            id: badge.id,
            title: badge.title,
            description: badge.description,
            score: badge.score,
            is_active: badge.is_active,
            earned_at: badge.earned_at,
        }
    }
}

#[instrument(skip(badge_use_cases))]
async fn read_badges(
    auth: AuthenticatedUser,
    State(badge_use_cases): State<Arc<BadgeUseCases>>,
) -> AppResult<impl IntoResponse> {
    let badges = badge_use_cases.read_all(&auth.public_key).await?;
    Ok((
        StatusCode::OK,
        Json(
            badges
                .into_iter()
                .map(BadgeResponse::from)
                .collect::<Vec<_>>(),
        ),
    ))
}
