use crate::adapters::http::app_state::AppState;
use crate::app_error::AppResult;
use crate::entities::leaderboard_entry::LeaderboardEntryDto;
use crate::use_cases::leaderboard::LeaderboardUseCases;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

pub fn public_router() -> Router<AppState> {
    Router::new().route("/", get(get_leaderboard))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardQueryParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_page() -> u32 {
    1
}
fn default_limit() -> u32 {
    10
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
            public_key: entry.masked_public_key,
            rank: entry.rank,
            total_score: entry.total_score,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardResponse {
    leaderboard: Vec<LeaderboardEntryResponse>,
    pagination: PaginationResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaginationResponse {
    page: u32,
    limit: u32,
    total: u32,
}

#[instrument(skip(leaderboard_use_cases))]
async fn get_leaderboard(
    Query(params): Query<LeaderboardQueryParams>,
    State(leaderboard_use_cases): State<Arc<LeaderboardUseCases>>,
) -> AppResult<impl IntoResponse> {
    let page = if params.page == 0 { 1 } else { params.page };
    let limit = if params.limit == 0 || params.limit > 100 {
        10
    } else {
        params.limit
    };

    let (entries, total) = leaderboard_use_cases.get_leaderboard(page, limit).await?;

    let leaderboard_response = entries
        .into_iter()
        .map(LeaderboardEntryResponse::from)
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(LeaderboardResponse {
            leaderboard: leaderboard_response,
            pagination: PaginationResponse { page, limit, total },
        }),
    ))
}
