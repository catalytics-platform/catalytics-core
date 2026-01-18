use crate::adapters::http::app_state::AppState;
use crate::adapters::http::middleware::auth::AuthenticatedUser;
use crate::adapters::http::middleware::auth_middleware;
use crate::app_error::AppResult;
use crate::entities::leaderboard_entry::LeaderboardEntryDto;
use crate::use_cases::leaderboard::LeaderboardUseCases;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router, middleware};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

pub fn private_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_user_leaderboard))
        .route("/list", get(get_leaderboard_list))
        .layer(middleware::from_fn(auth_middleware))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardQueryParams {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardEntryResponse {
    public_key: String,
    rank: u32,
    total_score: i32,
}

impl From<LeaderboardEntryDto> for LeaderboardEntryResponse {
    fn from(entry: LeaderboardEntryDto) -> Self {
        Self {
            public_key: mask_public_key(&entry.public_key),
            rank: entry.rank,
            total_score: entry.total_score,
        }
    }
}

fn mask_public_key(public_key: &str) -> String {
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

fn calculate_user_page(user_rank: u32, limit: u32) -> u32 {
    if user_rank == 0 {
        return 1;
    }
    ((user_rank - 1) / limit) + 1
}

fn is_user_on_page(user_rank: u32, page: u32, limit: u32) -> bool {
    if user_rank == 0 {
        return false;
    }
    let page_start = (page - 1) * limit + 1;
    let page_end = page * limit;
    user_rank >= page_start && user_rank <= page_end
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardResponse {
    leaderboard: Vec<LeaderboardEntryResponse>,
    pagination: PaginationResponse,
    user_context: UserContextResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaginationResponse {
    page: u32,
    limit: u32,
    total: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserContextResponse {
    rank: u32,
    total_score: i32,
    is_on_current_page: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserLeaderboardResponse {
    rank: u32,
    total_score: i32,
}

#[instrument(skip(leaderboard_use_cases))]
async fn get_leaderboard_list(
    auth: AuthenticatedUser,
    Query(params): Query<LeaderboardQueryParams>,
    State(leaderboard_use_cases): State<Arc<LeaderboardUseCases>>,
) -> AppResult<impl IntoResponse> {
    let user_entry = leaderboard_use_cases
        .get_user_leaderboard_entry(&auth.public_key)
        .await?;

    let (user_rank, user_score) = match &user_entry {
        Some(entry) => (entry.rank, entry.total_score),
        None => (0, 0),
    };

    let default_limit = 10;
    let limit = params.limit.unwrap_or(default_limit);

    let limit = if limit == 0 || limit > 100 { 10 } else { limit };

    let page = match params.page {
        Some(p) => {
            if p == 0 {
                1
            } else {
                p
            }
        }
        None => calculate_user_page(user_rank, limit),
    };

    let (entries, total) = leaderboard_use_cases.get_leaderboard(page, limit).await?;

    let leaderboard_response = entries
        .into_iter()
        .map(LeaderboardEntryResponse::from)
        .collect::<Vec<_>>();

    let is_user_on_page = is_user_on_page(user_rank, page, limit);

    Ok((
        StatusCode::OK,
        Json(LeaderboardResponse {
            leaderboard: leaderboard_response,
            pagination: PaginationResponse { page, limit, total },
            user_context: UserContextResponse {
                rank: user_rank,
                total_score: user_score,
                is_on_current_page: is_user_on_page,
            },
        }),
    ))
}

#[instrument(skip(leaderboard_use_cases))]
async fn get_user_leaderboard(
    auth: AuthenticatedUser,
    State(leaderboard_use_cases): State<Arc<LeaderboardUseCases>>,
) -> AppResult<impl IntoResponse> {
    let user_entry = leaderboard_use_cases
        .get_user_realtime_leaderboard_entry(&auth.public_key)
        .await?;

    let (user_rank, user_score) = match &user_entry {
        Some(entry) => (entry.rank, entry.total_score),
        None => (0, 0),
    };

    Ok((
        StatusCode::OK,
        Json(UserLeaderboardResponse {
            rank: user_rank,
            total_score: user_score,
        }),
    ))
}
